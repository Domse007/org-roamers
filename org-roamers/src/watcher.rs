use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use notify::{
    event::{AccessKind, CreateKind, ModifyKind, RemoveKind},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};

use crate::{
    diff,
    server::types::{RoamID, RoamLink, RoamNode},
    ServerState,
};

#[derive(Debug, Clone)]
pub enum OrgWatcherEvent {
    Create(PathBuf),
    Modify(PathBuf),
    Remove(PathBuf),
}

#[derive(Debug, Clone)]
pub struct GraphUpdate {
    pub new_nodes: Vec<RoamNode>,
    pub updated_nodes: Vec<RoamNode>,
    pub new_links: Vec<RoamLink>,
    pub removed_nodes: Vec<RoamID>,
    pub removed_links: Vec<RoamLink>,
}

impl GraphUpdate {
    pub fn new() -> Self {
        Self {
            new_nodes: Vec::new(),
            updated_nodes: Vec::new(),
            new_links: Vec::new(),
            removed_nodes: Vec::new(),
            removed_links: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.new_nodes.is_empty()
            && self.updated_nodes.is_empty()
            && self.new_links.is_empty()
            && self.removed_nodes.is_empty()
            && self.removed_links.is_empty()
    }
}

pub struct OrgWatcher {
    path: PathBuf,
    receiver: Receiver<Result<Event>>,
    watcher: RecommendedWatcher,
    pending_events: HashSet<PathBuf>,
    debounce_duration: Duration,
}

impl OrgWatcher {
    /// Process accumulated events and return changes
    pub fn process_events(
        &mut self,
        state: &mut ServerState,
    ) -> anyhow::Result<Option<GraphUpdate>> {
        // Collect events for debouncing
        let mut collected_events = Vec::new();

        // Non-blocking receive to collect all pending events
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    if let Some(org_events) = self.extract_org_events(event)? {
                        collected_events.extend(org_events);
                    }
                }
                Err(e) => tracing::warn!("Watcher event error: {}", e),
            }
        }

        if collected_events.is_empty() {
            return Ok(None);
        }

        // Debounce: add to pending and wait
        for event in collected_events {
            match &event {
                OrgWatcherEvent::Create(path)
                | OrgWatcherEvent::Modify(path)
                | OrgWatcherEvent::Remove(path) => {
                    self.pending_events.insert(path.clone());
                }
            }
        }

        // Wait for debounce period
        thread::sleep(self.debounce_duration);

        // Collect any additional events that came in during debounce
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    if let Some(org_events) = self.extract_org_events(event)? {
                        for event in org_events {
                            match &event {
                                OrgWatcherEvent::Create(path)
                                | OrgWatcherEvent::Modify(path)
                                | OrgWatcherEvent::Remove(path) => {
                                    self.pending_events.insert(path.clone());
                                }
                            }
                        }
                    }
                }
                Err(e) => tracing::warn!("Watcher event error during debounce: {}", e),
            }
        }

        // Process all pending events, filtering out files currently being processed
        let mut graph_update = GraphUpdate::new();
        let events_to_process: Vec<_> = self.pending_events.drain().collect();

        for path in events_to_process {
            // Skip files that are currently being processed by the application
            if self.should_ignore_file(&path, state) {
                tracing::debug!(
                    "Ignoring file watcher event for file being processed: {:?}",
                    path
                );
                continue;
            }

            match self.process_file_change(state, &path)? {
                Some(update) => self.merge_updates(&mut graph_update, update),
                None => continue,
            }
        }

        if graph_update.is_empty() {
            Ok(None)
        } else {
            Ok(Some(graph_update))
        }
    }

    /// Check if a file should be ignored due to being currently processed
    fn should_ignore_file(&self, file_path: &PathBuf, state: &ServerState) -> bool {
        state.dynamic_state.is_file_being_processed(file_path)
    }

    fn extract_org_events(&self, event: Event) -> anyhow::Result<Option<Vec<OrgWatcherEvent>>> {
        let mut org_events = vec![];

        let process_paths = |paths: Vec<PathBuf>,
                             event_type: fn(PathBuf) -> OrgWatcherEvent|
         -> Vec<OrgWatcherEvent> {
            paths
                .into_iter()
                .filter(|path| {
                    !path.is_dir() && path.extension().map(|ext| ext == "org").unwrap_or(false)
                })
                .map(event_type)
                .collect()
        };

        match event.kind {
            EventKind::Create(CreateKind::File) => {
                org_events.extend(process_paths(event.paths, OrgWatcherEvent::Create));
            }
            EventKind::Modify(ModifyKind::Data(_)) => {
                org_events.extend(process_paths(event.paths, OrgWatcherEvent::Modify));
            }
            EventKind::Remove(RemoveKind::File) => {
                org_events.extend(process_paths(event.paths, OrgWatcherEvent::Remove));
            }
            EventKind::Access(AccessKind::Close(_)) => {
                org_events.extend(process_paths(event.paths, OrgWatcherEvent::Modify));
            }
            other => {
                tracing::debug!("Unhandled event: {other:?}: {:?}", event.paths);
                return Ok(None);
            }
        }

        Ok(if org_events.is_empty() {
            None
        } else {
            Some(org_events)
        })
    }

    fn process_file_change(
        &self,
        state: &mut ServerState,
        path: &PathBuf,
    ) -> anyhow::Result<Option<GraphUpdate>> {
        let mut update = GraphUpdate::new();

        if path.exists() {
            // File was created or modified
            tracing::info!("Processing file change: {:?}", path);

            // Clear dynamic state before processing
            state.dynamic_state.updated_nodes.clear();
            state.dynamic_state.updated_links.clear();

            // Update the database
            if let Err(err) = diff::diff(state, path) {
                tracing::error!("Error processing file changes for {:?}: {}", path, err);
                return Ok(None);
            }

            // Use the changes detected by diff::diff
            update
                .new_nodes
                .extend(state.dynamic_state.updated_nodes.clone());
            update
                .new_links
                .extend(state.dynamic_state.updated_links.clone());

            tracing::info!(
                "File change processed: {} new nodes, {} new links",
                update.new_nodes.len(),
                update.new_links.len()
            );
        } else {
            // File was removed
            tracing::info!("Processing file removal: {:?}", path);

            // Get nodes that will be removed
            let nodes_to_remove = self.get_nodes_for_file(state, path);
            let links_to_remove = self.get_links_for_file(state, path);

            // Remove from database
            if let Err(err) = self.remove_file_from_db(state, path) {
                tracing::error!("Error removing file from database {:?}: {}", path, err);
                return Ok(None);
            }

            // Add to removed lists
            update
                .removed_nodes
                .extend(nodes_to_remove.into_iter().map(|n| n.id));
            update.removed_links.extend(links_to_remove);
        }

        Ok(if update.is_empty() {
            None
        } else {
            Some(update)
        })
    }

    fn get_nodes_for_file(&self, state: &ServerState, file_path: &PathBuf) -> Vec<RoamNode> {
        let file_str = file_path.to_string_lossy();
        let query = "SELECT id, title FROM nodes WHERE file = ?1";

        state
            .sqlite
            .query_many(query, [&file_str], |row| {
                Ok(RoamNode {
                    id: row.get::<usize, String>(0)?.into(),
                    title: row.get::<usize, String>(1)?.into(),
                    parent: "".into(),
                    num_links: 0, // Will be calculated later if needed
                })
            })
            .unwrap_or_default()
    }

    fn get_links_for_file(&self, state: &ServerState, file_path: &PathBuf) -> Vec<RoamLink> {
        let file_str = file_path.to_string_lossy();
        let query = r#"
            SELECT DISTINCT l.source, l.dest
            FROM links l
            JOIN nodes n ON (l.source = n.id OR l.dest = n.id)
            WHERE n.file = ?1 AND l.type = 'id'
        "#;

        state
            .sqlite
            .query_many(query, [&file_str], |row| {
                Ok(RoamLink {
                    from: row.get::<usize, String>(0)?.into(),
                    to: row.get::<usize, String>(1)?.into(),
                })
            })
            .unwrap_or_default()
    }

    fn remove_file_from_db(
        &self,
        state: &mut ServerState,
        file_path: &PathBuf,
    ) -> anyhow::Result<()> {
        let file_str = file_path.to_string_lossy();

        // Remove links first (due to foreign key constraints)
        let remove_links_query = r#"
            DELETE FROM links
            WHERE source IN (SELECT id FROM nodes WHERE file = ?1)
               OR dest IN (SELECT id FROM nodes WHERE file = ?1)
        "#;
        state.sqlite.execute(remove_links_query, [&file_str])?;

        // Remove nodes
        let remove_nodes_query = "DELETE FROM nodes WHERE file = ?1";
        state.sqlite.execute(remove_nodes_query, [&file_str])?;

        Ok(())
    }

    fn merge_updates(&self, target: &mut GraphUpdate, source: GraphUpdate) {
        target.new_nodes.extend(source.new_nodes);
        target.updated_nodes.extend(source.updated_nodes);
        target.new_links.extend(source.new_links);
        target.removed_nodes.extend(source.removed_nodes);
        target.removed_links.extend(source.removed_links);
    }
}

impl Drop for OrgWatcher {
    fn drop(&mut self) {
        if let Err(e) = self.watcher.unwatch(&self.path) {
            tracing::warn!("Failed to unwatch path {:?}: {}", self.path, e);
        }
    }
}

/// Construct a new watcher with WebSocket broadcasting capability
pub fn watcher(path: PathBuf) -> anyhow::Result<OrgWatcher> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(path.as_path(), RecursiveMode::Recursive)?;

    Ok(OrgWatcher {
        path,
        receiver: rx,
        watcher,
        pending_events: HashSet::new(),
        debounce_duration: Duration::from_millis(500), // 500ms debounce
    })
}

/// Start the watcher runtime with WebSocket broadcasting
pub fn start_watcher_runtime(
    app_state: Arc<Mutex<(crate::ServerState, Arc<Mutex<bool>>)>>,
    watch_path: PathBuf,
    runtime_handle: Option<tokio::runtime::Handle>,
) -> anyhow::Result<JoinHandle<()>> {
    let mut watcher = watcher(watch_path.clone())?;

    let handle = thread::spawn(move || {
        tracing::info!("File watcher started for: {:?}", watch_path);

        loop {
            // Process events every 100ms
            thread::sleep(Duration::from_millis(100));

            let (broadcaster, graph_update) = {
                let mut state_guard = match app_state.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        tracing::error!("Failed to acquire app state lock: {}", e);
                        continue;
                    }
                };

                let (ref mut server_state, _) = *state_guard;
                let broadcaster = server_state.websocket_broadcaster.clone();

                match watcher.process_events(server_state) {
                    Ok(update) => (broadcaster, update),
                    Err(e) => {
                        tracing::error!("Error processing file watcher events: {}", e);
                        continue;
                    }
                }
            }; // Lock is released here

            // Broadcast updates if any
            if let Some(update) = graph_update {
                tracing::info!("Broadcasting graph update: {} new nodes, {} updated nodes, {} new links, {} removed nodes, {} removed links",
                    update.new_nodes.len(),
                    update.updated_nodes.len(),
                    update.new_links.len(),
                    update.removed_nodes.len(),
                    update.removed_links.len()
                );

                // Broadcast detailed changes using runtime handle
                if let Some(ref handle) = runtime_handle {
                    handle.spawn(async move {
                        broadcaster
                            .broadcast_graph_update(
                                update.new_nodes,
                                update.updated_nodes,
                                update.new_links,
                                update.removed_nodes,
                                update.removed_links,
                            )
                            .await;
                    });
                } else {
                    tracing::warn!(
                        "No Tokio runtime handle available, WebSocket broadcast skipped"
                    );
                }
            }
        }
    });

    Ok(handle)
}

/// Legacy function for backward compatibility - now uses WebSocket broadcasting
pub fn default_watcher_runtime(
    app_state: Arc<Mutex<(crate::ServerState, Arc<Mutex<bool>>)>>,
    _watcher: OrgWatcher, // Deprecated parameter
    path: PathBuf,
) -> JoinHandle<()> {
    tracing::warn!("default_watcher_runtime is deprecated, use start_watcher_runtime instead");
    start_watcher_runtime(app_state, path, None).unwrap_or_else(|e| {
        tracing::error!("Failed to start watcher runtime: {}", e);
        thread::spawn(|| {}) // Return dummy handle
    })
}

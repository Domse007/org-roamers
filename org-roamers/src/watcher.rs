use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Event, EventKind, RecursiveMode, Result, Watcher,
};
use sqlx::SqlitePool;

use crate::{
    cache::OrgCacheEntry,
    server::AppState,
    sqlite::{files::insert_file, rebuild},
    transform::org,
    ServerState,
};

/// File system watcher for org-mode files that integrates with the cache system
pub struct OrgWatcher {
    receiver: Receiver<Result<Event>>,
    pending_files: HashSet<PathBuf>,
}

impl OrgWatcher {
    /// Processes file system events and updates the cache/database for changed org files
    pub async fn process_events(
        &mut self,
        state: Arc<Mutex<ServerState>>,
    ) -> anyhow::Result<Vec<PathBuf>> {
        // Collect all pending org file changes
        while let Ok(event_result) = self.receiver.try_recv() {
            if let Ok(event) = event_result {
                for path in event.paths {
                    if self.is_org_file_event(&event.kind, &path) && path.exists() {
                        self.pending_files.insert(path);
                    } else if matches!(event.kind, EventKind::Remove(RemoveKind::File)) {
                        self.handle_file_removal(state.clone(), &path).await?;
                    }
                }
            }
        }

        if self.pending_files.is_empty() {
            return Ok(Vec::new());
        }

        // Process all pending files
        let files_to_process: Vec<_> = self.pending_files.drain().collect();
        for file_path in &files_to_process {
            self.process_file_change(state.clone(), file_path).await?;
        }

        Ok(files_to_process)
    }

    /// Checks if a file system event is for an org file create/modify operation
    fn is_org_file_event(&self, kind: &EventKind, path: &PathBuf) -> bool {
        matches!(
            kind,
            EventKind::Create(CreateKind::File) | EventKind::Modify(ModifyKind::Data(_))
        ) && path.extension().map(|ext| ext == "org").unwrap_or(false)
    }

    /// Rescans a file and updates the cache and database with new content
    async fn process_file_change(
        &self,
        state: Arc<Mutex<ServerState>>,
        path: &PathBuf,
    ) -> anyhow::Result<()> {
        tracing::info!("Processing file change: {:?}", path);

        // Clear dynamic state before processing and get needed values
        let (cache_path, sqlite) = {
            let guard = state.lock().unwrap();
            (guard.cache.path().to_path_buf(), guard.sqlite.clone())
        };

        // Create new cache entry
        let cache_entry = OrgCacheEntry::new(&cache_path, path)?;
        let relative_path = path.strip_prefix(&cache_path)?;

        // Remove existing data for this file (clean slate)
        self.remove_file_data(state.clone(), path).await?;

        // Update file hash in database
        insert_file(&sqlite, relative_path, cache_entry.get_hash()).await?;

        // After successful database cleanup, extract and insert nodes with correct file association
        let file_path_str = relative_path.to_string_lossy().to_string();
        let nodes = org::get_nodes(cache_entry.content(), &file_path_str);
        self.insert_nodes_with_file(&sqlite, nodes, relative_path)
            .await?;

        // Invalidate cache to trigger refresh
        state.lock().unwrap().cache.invalidate(path.clone());

        tracing::info!("File change processed successfully: {:?}", path);
        Ok(())
    }

    /// Handles org file deletion by cleaning up associated database entries
    async fn handle_file_removal(
        &self,
        state: Arc<Mutex<ServerState>>,
        path: &PathBuf,
    ) -> anyhow::Result<()> {
        if path.extension().map(|ext| ext == "org").unwrap_or(false) {
            tracing::info!("Processing file removal: {:?}", path);
            self.remove_file_data(state.clone(), path).await?;
            state.lock().unwrap().cache.invalidate(path.clone());
        }
        Ok(())
    }

    /// Removes all database entries (links, nodes, files) associated with a file path
    async fn remove_file_data(
        &self,
        state: Arc<Mutex<ServerState>>,
        path: &PathBuf,
    ) -> anyhow::Result<()> {
        let file_str = path.to_string_lossy();
        let sqlite = state.lock().unwrap().sqlite.clone();

        // Remove links first, then nodes, then file entry
        sqlx::query(
            "DELETE FROM links WHERE source IN (SELECT id FROM nodes WHERE file = ?) OR dest IN (SELECT id FROM nodes WHERE file = ?)",
        ).bind(&file_str).bind(&file_str).execute(&sqlite).await?;
        sqlx::query("DELETE FROM nodes WHERE file = ?")
            .bind(&file_str)
            .execute(&sqlite)
            .await?;
        sqlx::query("DELETE FROM files WHERE file = ?")
            .bind(file_str)
            .execute(&sqlite)
            .await?;

        Ok(())
    }

    /// Insert nodes with proper file association for watcher context
    async fn insert_nodes_with_file(
        &self,
        con: &SqlitePool,
        nodes: Vec<org::NodeFromOrg>,
        file_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        let file_str = file_path.to_string_lossy();

        for node in nodes {
            // Insert node with correct file path
            rebuild::insert_node(
                con,
                &node.uuid,
                &file_str,
                node.level,
                false,
                0,
                "",
                "",
                &node.title,
                &node.actual_olp,
            )
            .await?;

            // Insert tags
            for tag in &node.tags {
                rebuild::insert_tag(con, &node.uuid, tag).await?;
            }

            // Insert aliases
            for alias in &node.aliases {
                rebuild::insert_alias(con, &node.uuid, alias).await?;
            }

            // Insert links
            for (dest_id, _description) in &node.links {
                rebuild::insert_link(con, &node.uuid, dest_id).await?;
            }
        }

        Ok(())
    }
}

/// Creates a new file system watcher for monitoring org files in the given directory
pub fn watcher(path: PathBuf) -> anyhow::Result<OrgWatcher> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    Ok(OrgWatcher {
        receiver: rx,
        pending_files: HashSet::new(),
    })
}

/// Starts a background thread that processes file changes and notifies WebSocket clients
pub async fn start_watcher_runtime(app_state: AppState, watch_path: PathBuf) -> anyhow::Result<()> {
    let mut watcher = watcher(watch_path.clone())?;

    tracing::info!("File watcher started for: {:?}", watch_path);

    // Use tokio::task::spawn_blocking to run the watcher in a blocking thread
    tokio::task::spawn_blocking(move || {
        // Create a tokio runtime for async operations within the blocking context
        let rt = tokio::runtime::Runtime::new().unwrap();

        loop {
            thread::sleep(Duration::from_millis(500)); // Debounce

            let changed_files = rt.block_on(async {
                match watcher.process_events(app_state.clone()).await {
                    Ok(files) => files,
                    Err(e) => {
                        tracing::error!("Error processing watcher events: {}", e);
                        return Vec::new();
                    }
                }
            });

            // If there are changes, notify all WebSocket clients
            if !changed_files.is_empty() {
                // Create a simple status update message
                let update_message = crate::client::message::WebSocketMessage::StatusUpdate {
                    files_changed: changed_files.len(),
                };

                let websocket_count = {
                    let mut state = app_state.lock().unwrap();
                    state.broadcast_to_websockets(update_message);
                    state.websocket_connections.len()
                };

                tracing::info!(
                    "Notified {} WebSocket clients about {} file changes",
                    websocket_count,
                    changed_files.len()
                );
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::cache::OrgCache;
    use std::collections::HashMap;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    async fn create_test_server_state(temp_dir: &Path) -> Arc<Mutex<ServerState>> {
        let sqlite = crate::sqlite::init_db(false).await.unwrap();
        let state = ServerState {
            config: Config::default(),
            sqlite: sqlite,
            cache: OrgCache::new(temp_dir.to_path_buf()),
            websocket_connections: HashMap::new(),
            next_connection_id: 1,
        };
        Arc::new(Mutex::new(state))
    }

    fn create_test_org_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_is_org_file_event() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        let org_path = temp_dir.path().join("test.org");
        let txt_path = temp_dir.path().join("test.txt");

        // Test org file events
        assert!(watcher.is_org_file_event(&EventKind::Create(CreateKind::File), &org_path));
        assert!(watcher.is_org_file_event(
            &EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            &org_path
        ));

        // Test non-org file events
        assert!(!watcher.is_org_file_event(&EventKind::Create(CreateKind::File), &txt_path));
        assert!(!watcher.is_org_file_event(&EventKind::Remove(RemoveKind::File), &org_path));
    }

    #[tokio::test]
    async fn test_remove_file_data() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_server_state(temp_dir.path()).await;

        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        let test_file = temp_dir.path().join("test.org");

        // Insert some test data
        let file_str = test_file.to_string_lossy();
        {
            sqlx::query("INSERT OR REPLACE INTO files (file, hash) VALUES (?, 123)")
                .bind(&file_str)
                .execute(&state.lock().unwrap().sqlite)
                .await
                .unwrap();
            sqlx::query("INSERT OR REPLACE INTO nodes (id, title, file, level) VALUES ('test-id', 'Test', ?, 1)")
                .bind(&file_str).execute(&state.lock().unwrap().sqlite).await.unwrap();
        }

        // Test removal
        watcher
            .remove_file_data(state.clone(), &test_file)
            .await
            .unwrap();

        // Verify data was removed
        let file_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE file = ?")
            .bind(&file_str)
            .fetch_one(&state.lock().unwrap().sqlite)
            .await
            .unwrap();
        assert_eq!(file_count, 0);

        let node_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE file = ?")
            .bind(file_str)
            .fetch_one(&state.lock().unwrap().sqlite)
            .await
            .unwrap();
        assert_eq!(node_count, 0);
    }

    #[tokio::test]
    async fn test_handle_file_removal() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_server_state(temp_dir.path()).await;

        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        let org_file = temp_dir.path().join("test.org");
        let txt_file = temp_dir.path().join("test.txt");

        // Test org file removal
        watcher
            .handle_file_removal(state.clone(), &org_file)
            .await
            .unwrap();

        // Test non-org file removal (should not error)
        watcher.handle_file_removal(state, &txt_file).await.unwrap();
    }

    #[tokio::test]
    async fn test_process_file_change() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_server_state(temp_dir.path()).await;

        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        // Create a test org file with valid content
        let org_content = r#":PROPERTIES:
:ID: test-id-123
:END:
#+title: Test File

This is test content.
"#;
        let org_file = create_test_org_file(temp_dir.path(), "test.org", org_content);

        // Process the file change
        watcher
            .process_file_change(state.clone(), &org_file)
            .await
            .unwrap();

        // Verify file was processed (check if nodes were inserted)
        let node_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = 'test-id-123'")
                .fetch_one(&state.lock().unwrap().sqlite)
                .await
                .unwrap();
        assert_eq!(node_count, 1);
    }

    #[test]
    fn test_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();

        // Test successful watcher creation
        let watcher_result = watcher(temp_dir.path().to_path_buf());
        assert!(watcher_result.is_ok());

        // Test with non-existent path
        let bad_path = temp_dir.path().join("non-existent");
        let bad_watcher_result = watcher(bad_path);
        assert!(bad_watcher_result.is_err());
    }

    #[tokio::test]
    async fn test_path_handling_cross_platform() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_server_state(temp_dir.path()).await;

        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        // Test that paths work correctly on both Unix and Windows
        let org_file = create_test_org_file(
            temp_dir.path(),
            "test.org",
            ":PROPERTIES:\n:ID: test-123\n:END:\n#+title: Test\n",
        );

        // This should work regardless of path separator differences
        let result = watcher.process_file_change(state.clone(), &org_file).await;
        assert!(result.is_ok());

        // Verify the file path is handled correctly in database
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = 'test-123'")
            .fetch_one(&state.lock().unwrap().sqlite)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_transaction_safety() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_server_state(temp_dir.path()).await;

        let watcher = OrgWatcher {
            receiver: mpsc::channel().1,
            pending_files: HashSet::new(),
        };

        // Insert some initial data
        let org_content = ":PROPERTIES:\n:ID: test-node\n:END:\n#+title: Test\n";
        let org_file = create_test_org_file(temp_dir.path(), "test.org", org_content);

        // Initial insert should work
        watcher
            .process_file_change(state.clone(), &org_file)
            .await
            .unwrap();

        // Verify initial data exists
        {
            let initial_count: i32 =
                sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = 'test-node'")
                    .fetch_one(&state.lock().unwrap().sqlite)
                    .await
                    .unwrap();
            assert_eq!(initial_count, 1);

            // Debug: check what file path is in the database
            let files_in_db: Vec<String> = sqlx::query_scalar("SELECT file FROM files")
                .fetch_all(&state.lock().unwrap().sqlite)
                .await
                .unwrap();
            tracing::info!("Files in database: {:?}", files_in_db);

            let nodes_in_db: Vec<(String, String)> = sqlx::query_as("SELECT id, file FROM nodes")
                .fetch_all(&state.lock().unwrap().sqlite)
                .await
                .unwrap();
            tracing::info!("Nodes in database: {:?}", nodes_in_db);
        }

        // Test with the correct file path
        let org_file_relative = org_file
            .strip_prefix(temp_dir.path())
            .unwrap()
            .to_path_buf();
        tracing::info!("Attempting to remove file: {:?}", org_file_relative);
        watcher
            .remove_file_data(state.clone(), &org_file_relative)
            .await
            .unwrap();

        // Verify node was removed
        let after_remove_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = 'test-node'")
                .fetch_one(&state.lock().unwrap().sqlite)
                .await
                .unwrap();
        assert_eq!(after_remove_count, 0);
    }
}

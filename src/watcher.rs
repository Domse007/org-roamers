use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use notify::{
    event::{AccessKind, CreateKind, ModifyKind, RemoveKind},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use rusqlite::Connection;

use crate::{transform::org::get_nodes_from_file, ServerState};

pub enum OrgWatcherEvent {
    Create(PathBuf),
    Modify(PathBuf),
    Remove(PathBuf),
}

pub struct OrgWatcher {
    path: PathBuf,
    receiver: Receiver<Result<Event>>,
    watcher: RecommendedWatcher,
    changes: Arc<Mutex<bool>>,
}

impl OrgWatcher {
    /// Get an event from the watcher.
    pub fn handle(&mut self, base: PathBuf) -> anyhow::Result<Option<Vec<OrgWatcherEvent>>> {
        let event = self.receiver.recv()??;
        let mut org_events = vec![];
        let create_events = |buf: &mut Vec<OrgWatcherEvent>,
                             paths: Vec<PathBuf>,
                             new: fn(PathBuf) -> OrgWatcherEvent| {
            for path in paths {
                if path.is_dir() {
                    break;
                }
                if let Some(ext) = path.extension() {
                    if ext != "org" {
                        break;
                    }
                } else {
                    break;
                }
                let mut expanded_path = base.clone();
                expanded_path.push(path);
                buf.push(new(expanded_path));
            }
        };
        match event.kind {
            EventKind::Create(kind) => {
                if let CreateKind::File = kind {
                    create_events(&mut org_events, event.paths, |p| OrgWatcherEvent::Create(p));
                }
            }
            EventKind::Modify(kind) => {
                if let ModifyKind::Data(_) = kind {
                    create_events(&mut org_events, event.paths, |p| OrgWatcherEvent::Modify(p));
                }
            }
            EventKind::Remove(kind) => {
                if let RemoveKind::File = kind {
                    create_events(&mut org_events, event.paths, |p| OrgWatcherEvent::Remove(p));
                }
            }
            EventKind::Access(kind) => {
                if let AccessKind::Close(_) = kind {
                    create_events(&mut org_events, event.paths, |p| OrgWatcherEvent::Modify(p));
                }
            }
            other => {
                tracing::info!("Unhandled event: {other:?}: {:?}", event.paths);
                return Ok(None);
            }
        }
        Ok(Some(org_events))
    }

    pub fn set_changes(&mut self, new: bool) {
        *self.changes.lock().unwrap() = new;
    }
}

impl Drop for OrgWatcher {
    fn drop(&mut self) {
        self.watcher.unwatch(self.path.as_path()).unwrap();
    }
}

/// Construct a new watcher.
pub fn watcher(path: PathBuf) -> anyhow::Result<(OrgWatcher, Arc<Mutex<bool>>)> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    let changes = Arc::new(Mutex::new(false));

    watcher.watch(path.as_path(), RecursiveMode::Recursive)?;

    Ok((
        OrgWatcher {
            path,
            receiver: rx,
            watcher,
            changes: changes.clone(),
        },
        changes,
    ))
}

fn get_nodes_from_event_path(db: &mut Connection, path: PathBuf) -> anyhow::Result<()> {
    let nodes = match get_nodes_from_file(path) {
        Ok(nodes) => nodes,
        Err(err) => {
            tracing::error!("Some error occured: {err}");
            return Ok(());
        }
    };
    for node in nodes {
        if let Err(err) = node.insert_into(db, true) {
            tracing::error!(
                "An error occured while updating {} ({}): {}",
                node.uuid,
                node.title,
                err
            );
        }
    }
    Ok(())
}

fn handle_event(db: Arc<Mutex<ServerState>>, event: OrgWatcherEvent) -> anyhow::Result<()> {
    match event {
        OrgWatcherEvent::Create(path) => {
            if let Err(err) = db.lock().unwrap().sqlite.insert_files(&path) {
                tracing::error!("An error occured while adding {path:?} to the db: {err:?}");
                return Ok(());
            }
        }
        OrgWatcherEvent::Modify(path) => {
            tracing::info!("Processing modified {path:?}");
            get_nodes_from_event_path(db.lock().unwrap().sqlite.connection(), path)?
        }
        OrgWatcherEvent::Remove(_path) => {
            // TODO: broken...
            // get_nodes_from_event_path(db.lock().unwrap().sqlite.connection(), path)?
        }
    }
    return Ok(());
}

pub fn default_watcher_runtime(
    db: Arc<Mutex<ServerState>>,
    mut watcher: OrgWatcher,
    path: PathBuf,
) -> JoinHandle<()> {
    let err_handler =
        |err: Box<dyn std::error::Error>| tracing::error!("File watcher encountered error: {err}");
    let handle = thread::spawn(move || loop {
        let path = path.clone();
        match watcher.handle(path) {
            Ok(handle) => {
                if let Some(events) = handle {
                    for event in events {
                        match handle_event(db.clone(), event) {
                            Ok(_) => watcher.set_changes(true),
                            Err(err) => err_handler(err.into()),
                        }
                    }
                }
            }
            Err(err) => err_handler(err.into()),
        }
    });

    return handle;
}

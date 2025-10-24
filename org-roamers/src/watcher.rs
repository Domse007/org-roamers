use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    cache::OrgCacheEntry, client::message::WebSocketMessage, server::types::RoamID,
    sqlite::files::insert_file, transform::node_builder, ServerState,
};

pub async fn watcher(
    state: Arc<ServerState>,
    cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
    let path = state.cache.path().to_path_buf();
    let (tx, mut rx) = mpsc::channel(100);
    let rt = Handle::current();

    let mut debouncer = new_debouncer(
        Duration::from_secs(2),
        None,
        move |result: DebounceEventResult| {
            let tx = tx.clone();
            let rt = rt.clone();

            rt.spawn(async move {
                if let Err(e) = tx.send(result).await {
                    tracing::debug!("Failed to send watcher event: {}", e);
                }
            });
        },
    )?;

    debouncer.watch(&path, RecursiveMode::Recursive)?;

    tokio::spawn(async move {
        let _debouncer = debouncer;

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    tracing::info!("Watcher cancelled");
                    break;
                }
                Some(result) = rx.recv() => {
                    handle_watcher_event(result, &state).await;
                }
            }
        }

        tracing::info!("Watcher shutdown complete");
    });

    Ok(())
}

async fn handle_watcher_event(result: DebounceEventResult, state: &ServerState) {
    match result {
        Ok(events) => {
            let paths: Vec<PathBuf> = events
                .iter()
                .filter(|event| is_write_event(&event.kind))
                .flat_map(|e| e.paths.clone())
                .collect();

            let filtered = filter_org_files(paths);
            let mut files_updated = 0;

            for path in filtered {
                tracing::info!("File changed: {:?}", path);

                // Update both cache and database
                if let Err(e) = update_file(state, &path).await {
                    tracing::error!("Failed to update file {:?}: {}", path, e);
                } else {
                    files_updated += 1;
                }
            }

            // Notify all WebSocket clients about the changes
            if files_updated > 0 {
                let message = WebSocketMessage::StatusUpdate {
                    files_changed: files_updated,
                };
                state.broadcast_to_websockets(message);
                tracing::info!(
                    "Notified WebSocket clients: {} files changed",
                    files_updated
                );
            }
        }
        Err(errors) => {
            for error in errors {
                tracing::error!("Watcher error: {error}");
            }
        }
    }
}

async fn update_file(state: &ServerState, path: &PathBuf) -> anyhow::Result<()> {
    // Create new cache entry by reading the file
    let cache_entry = OrgCacheEntry::new(state.cache.path(), path)?;

    // Update database with file metadata
    insert_file(&state.sqlite, cache_entry.path(), cache_entry.get_hash()).await?;

    // Parse org content to extract nodes
    let file_path_str = cache_entry.path().to_string_lossy().to_string();
    let nodes = node_builder::get_nodes(cache_entry.content(), &file_path_str);

    // Collect node IDs
    let node_ids: Vec<RoamID> = nodes.iter().map(|n| n.uuid.clone().into()).collect();

    // Update cache with all nodes from this file
    state.cache.insert_many(&node_ids, cache_entry);

    // Update nodes in database
    node_builder::insert_nodes(&state.sqlite, nodes).await;

    tracing::info!("Updated file {:?} in cache and database", file_path_str);
    Ok(())
}

fn is_write_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(CreateKind::File)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Name(_))
            | EventKind::Modify(ModifyKind::Any)
            | EventKind::Remove(RemoveKind::File)
    )
}

fn filter_org_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|path| path.extension().map(|ext| ext == "org").unwrap_or(false))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filter_org_files() {
        let paths = vec![
            PathBuf::from("/org/test.pdf"),
            PathBuf::from("/org/test.org"),
            PathBuf::from("other.sorg"),
        ];
        let res = filter_org_files(paths);
        assert_eq!(res, vec![PathBuf::from("/org/test.org")]);
    }
}

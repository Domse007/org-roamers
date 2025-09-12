use std::path::Path;

use crate::{
    server::types::RoamLink, sqlite::rebuild, transform::org::get_nodes_from_file, ServerState,
};

pub fn diff<P: AsRef<Path>>(state: &mut ServerState, file: P) -> anyhow::Result<()> {
    let nodes = get_nodes_from_file(file)?;

    for node in nodes {
        // Check if this is a new node
        const NODE_CHECK_STMNT: &str = "SELECT id FROM nodes WHERE id = ?1;";
        let node_exists = state
            .sqlite
            .query_one(NODE_CHECK_STMNT, [&node.uuid], |row| {
                Ok(row.get::<usize, String>(0).unwrap())
            })
            .is_ok();

        // If it's a new node, insert it first
        if !node_exists {
            node.insert_into(&mut state.sqlite.connection())?;
            state.dynamic_state.updated_nodes.push(node.clone().into());
            state.dynamic_state.pending_reload = true;
            tracing::info!("Added new node: {:?} ({})", node.title, node.uuid);

            // Check if this node has a parent and create parent-child link
            if let Some(parent_id) = &node.parent {
                // Verify parent exists in database
                const PARENT_EXISTS_STMNT: &str = "SELECT id FROM nodes WHERE id = ?1;";
                let parent_exists = state
                    .sqlite
                    .query_one(PARENT_EXISTS_STMNT, [parent_id], |row| {
                        Ok(row.get::<usize, String>(0).unwrap())
                    })
                    .is_ok();

                if parent_exists {
                    // Check if parent-child link already exists
                    const PARENT_LINK_CHECK: &str =
                        "SELECT source, dest FROM links WHERE source = ?1 AND dest = ?2;";
                    let parent_link_exists = state
                        .sqlite
                        .query_one(PARENT_LINK_CHECK, [parent_id, &node.uuid], |row| {
                            Ok(row.get::<usize, String>(0).unwrap())
                        })
                        .is_ok();

                    if !parent_link_exists {
                        // Create parent -> child link
                        rebuild::insert_link(
                            &mut state.sqlite.connection(),
                            parent_id,
                            &node.uuid,
                        )?;
                        state.dynamic_state.updated_links.push(RoamLink {
                            from: parent_id.clone().into(),
                            to: node.uuid.clone().into(),
                        });
                        tracing::info!("Added parent-child link: {} -> {}", parent_id, node.uuid);
                    }
                } else {
                    tracing::debug!(
                        "Parent node {} does not exist for child {}",
                        parent_id,
                        node.uuid
                    );
                }
            }
        }

        // Process all links from this node
        for (dest, _) in &node.links {
            // Check if the destination node exists
            const DEST_CHECK_STMNT: &str = "SELECT id FROM nodes WHERE id = ?1;";
            let dest_exists = state
                .sqlite
                .query_one(DEST_CHECK_STMNT, [&dest], |row| {
                    Ok(row.get::<usize, String>(0).unwrap())
                })
                .is_ok();

            if !dest_exists {
                tracing::debug!(
                    "Skipping link to non-existent node: {} -> {}",
                    node.uuid,
                    dest
                );
                continue;
            }

            // Check if the link already exists
            const LINK_CHECK_STMNT: &str =
                "SELECT source, dest FROM links WHERE source = ?1 AND dest = ?2;";
            let link_exists = state
                .sqlite
                .query_one(LINK_CHECK_STMNT, [&node.uuid, &dest], |row| {
                    Ok(row.get::<usize, String>(0).unwrap())
                })
                .is_ok();

            if !link_exists {
                let source = &node.uuid;
                rebuild::insert_link(&mut state.sqlite.connection(), &source, &dest)?;
                state.dynamic_state.updated_links.push(RoamLink {
                    from: source.clone().into(),
                    to: dest.clone().into(),
                });
                tracing::info!("Added new link: {} -> {}", source, dest);
            } else {
                tracing::debug!("Link already exists: {} -> {}", node.uuid, dest);
            }
        }
    }

    tracing::info!(
        "Dynamic state after diff - nodes: {}, links: {}",
        state.dynamic_state.updated_nodes.len(),
        state.dynamic_state.updated_links.len()
    );

    Ok(())
}

use futures_util::StreamExt;
use sqlx::SqlitePool;

use crate::server::types::{GraphData, RoamID, RoamLink, RoamNode};
use crate::sqlite::olp;
use crate::transform::title::TitleSanitizer;

pub async fn get_graph_data(sqlite: &SqlitePool) -> GraphData {
    let title_sanitizer = |title: &str| {
        let sanitizer = TitleSanitizer::new();
        sanitizer.process(title)
    };

    let string_nodes = sqlx::query_as::<_, (String, String)>("SELECT id, title FROM nodes;")
        .fetch_all(sqlite)
        .await
        .unwrap();

    let mut nodes: Vec<RoamNode> = vec![];

    for node in string_nodes {
        let parent = olp::get_olp(sqlite, &node.0)
            .await
            .unwrap_or_default()
            .pop()
            .unwrap_or_default();
        let stmnt = "SELECT id FROM nodes WHERE title = ?";
        let parent_id: String = sqlx::query_scalar(stmnt)
            .bind(parent)
            .fetch_one(sqlite)
            .await
            .unwrap_or_default();
        nodes.push(RoamNode {
            title: title_sanitizer(&node.1).into(),
            id: node.0.to_string().into(),
            parent: parent_id.into(),
            num_links: 0,
        });
    }

    const STMNT: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = 'id'\n",
        "AND (dest = ? OR source = ?)"
    );

    for node in &mut nodes {
        // TODO: use count dumbass...
        let results: Vec<(String, String, String)> = sqlx::query_as(STMNT)
            .bind(node.id.id())
            .bind(node.id.id())
            .fetch_all(sqlite)
            .await
            .unwrap_or_default();
        node.num_links = results.len();
    }

    const ALL_LINKS: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = 'id';"
    );

    let mut links: Vec<RoamLink> = sqlx::query_as::<_, (String, String, String)>(ALL_LINKS)
        .fetch(sqlite)
        .filter_map(|res| async move {
            match res {
                Ok((source, dest, _)) => Some(RoamLink {
                    from: RoamID::from(source),
                    to: RoamID::from(dest),
                }),
                Err(_) => None,
            }
        })
        .collect()
        .await;

    // Add parent-child hierarchy links
    for node in &nodes {
        // Only add a link if the node has a non-empty parent
        if !node.parent.id().is_empty() {
            links.push(RoamLink {
                from: node.parent.clone(),
                to: node.id.clone(),
            });
        }
    }

    GraphData { nodes, links }
}

use futures_util::StreamExt;
use sqlx::SqlitePool;
use std::collections::HashSet;

use crate::server::types::{GraphData, RoamID, RoamLink, RoamNode};
use crate::sqlite::olp;
use crate::transform::title::TitleSanitizer;

pub async fn get_graph_data(
    sqlite: &SqlitePool,
    filter_tags: Option<Vec<String>>,
    exclude_tags: Option<Vec<String>>,
) -> GraphData {
    let title_sanitizer = |title: &str| {
        let sanitizer = TitleSanitizer::new();
        sanitizer.process(title)
    };

    let string_nodes = match (filter_tags, exclude_tags) {
        (None, None) => sqlx::query_as::<_, (String, String)>("SELECT id, title FROM nodes;")
            .fetch_all(sqlite)
            .await
            .unwrap(),
        (Some(tags), None) if tags.is_empty() => {
            sqlx::query_as::<_, (String, String)>("SELECT id, title FROM nodes;")
                .fetch_all(sqlite)
                .await
                .unwrap()
        }
        (None, Some(excl)) if !excl.is_empty() => {
            let placeholders = excl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT DISTINCT n.id, n.title FROM nodes n WHERE n.id NOT IN (SELECT node_id FROM tags WHERE tag IN ({}))",
                placeholders
            );
            let mut q = sqlx::query_as::<_, (String, String)>(&query);
            for tag in excl {
                q = q.bind(tag);
            }
            q.fetch_all(sqlite).await.unwrap()
        }
        (Some(incl), None) => {
            let placeholders = incl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT DISTINCT n.id, n.title FROM nodes n INNER JOIN tags t ON n.id = t.node_id WHERE t.tag IN ({})",
                placeholders
            );
            let mut q = sqlx::query_as::<_, (String, String)>(&query);
            for tag in incl {
                q = q.bind(tag);
            }
            q.fetch_all(sqlite).await.unwrap()
        }
        (incl_opt, Some(excl)) if !excl.is_empty() => {
            let mut query = String::from("SELECT DISTINCT n.id, n.title FROM nodes n");
            let mut bindings: Vec<String> = vec![];

            if let Some(incl) = incl_opt {
                if !incl.is_empty() {
                    let placeholders = incl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    query.push_str(&format!(
                        " INNER JOIN tags t ON n.id = t.node_id WHERE t.tag IN ({})",
                        placeholders
                    ));
                    bindings.extend(incl);
                    let excl_placeholders = excl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    query.push_str(&format!(
                        " AND n.id NOT IN (SELECT node_id FROM tags WHERE tag IN ({}))",
                        excl_placeholders
                    ));
                    bindings.extend(excl);
                } else {
                    let excl_placeholders = excl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    query.push_str(&format!(
                        " WHERE n.id NOT IN (SELECT node_id FROM tags WHERE tag IN ({}))",
                        excl_placeholders
                    ));
                    bindings.extend(excl);
                }
            } else {
                let excl_placeholders = excl.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                query.push_str(&format!(
                    " WHERE n.id NOT IN (SELECT node_id FROM tags WHERE tag IN ({}))",
                    excl_placeholders
                ));
                bindings.extend(excl);
            }

            let mut q = sqlx::query_as::<_, (String, String)>(&query);
            for tag in bindings {
                q = q.bind(tag);
            }
            q.fetch_all(sqlite).await.unwrap()
        }
        _ => sqlx::query_as::<_, (String, String)>("SELECT id, title FROM nodes;")
            .fetch_all(sqlite)
            .await
            .unwrap(),
    };

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

    let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.id().to_string()).collect();

    const ALL_LINKS: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = 'id';"
    );

    let mut links: Vec<RoamLink> = sqlx::query_as::<_, (String, String, String)>(ALL_LINKS)
        .fetch(sqlite)
        .filter_map(|res| {
            let node_ids = node_ids.clone();
            async move {
                match res {
                    Ok((source, dest, _)) => {
                        if node_ids.contains(&source) && node_ids.contains(&dest) {
                            Some(RoamLink {
                                from: RoamID::from(source),
                                to: RoamID::from(dest),
                            })
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
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

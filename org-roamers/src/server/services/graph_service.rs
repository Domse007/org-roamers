use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;

use crate::server::types::{GraphData, RoamLink, RoamNode};
use crate::sqlite::{helpers, olp};
use crate::transform::title::TitleSanitizer;
use crate::ServerState;

pub fn get_graph_data(db: &mut ServerState) -> GraphData {
    let title_sanitizer = |title: &str| {
        let sanitizer = TitleSanitizer::new();
        sanitizer.process(title)
    };

    let mut nodes = helpers::get_all_nodes(db.sqlite.connection(), ["id", "title"])
        .into_iter()
        .map(|e| {
            let parent = olp::get_olp(db.sqlite.connection(), &e[0])
                .unwrap_or_default()
                .pop()
                .unwrap_or_default();
            let stmnt = "SELECT title, id FROM nodes WHERE title = ?1;";
            let parent = db
                .sqlite
                .query_one(stmnt, [parent], |row| {
                    Ok(row.get::<usize, String>(1).unwrap())
                })
                .unwrap_or_default();
            RoamNode {
                title: title_sanitizer(&e[1]).into(),
                id: e[0].to_string().into(),
                parent: parent.into(),
                num_links: 0,
            }
        })
        .collect::<Vec<RoamNode>>();

    const STMNT: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = 'id'\n",
        "AND (dest = ?1 OR source = ?1)"
    );

    let mut stmnt = db.sqlite.connection().prepare(STMNT).unwrap();
    for node in &mut nodes {
        let num = stmnt.query([node.id.id()]).unwrap().count().unwrap();
        node.num_links = num;
    }

    drop(stmnt);

    const ALL_LINKS: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = 'id';"
    );

    let mut links: Vec<RoamLink> = db
        .sqlite
        .query_many(ALL_LINKS, [], |row| {
            Ok(RoamLink {
                from: row.get::<usize, String>(0).unwrap().into(),
                to: row.get::<usize, String>(1).unwrap().into(),
            })
        })
        .unwrap()
        .into_iter()
        .collect();

    const PARENT_STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?1;";

    for node in &nodes {
        if let Ok(parent_id) = db.sqlite.query_one(PARENT_STMNT, [&node.id.id()], |row| {
            row.get::<usize, String>(0)
        }) {
            links.push(RoamLink {
                from: parent_id.into(),
                to: node.id.clone(),
            });
        }
    }

    GraphData { nodes, links }
}

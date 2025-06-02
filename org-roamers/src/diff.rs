use std::path::Path;

use crate::{
    api::types::RoamLink, sqlite::rebuild, transform::org::get_nodes_from_file, ServerState,
};

pub fn diff<P: AsRef<Path>>(state: &mut ServerState, file: P) -> anyhow::Result<()> {
    let nodes = get_nodes_from_file(file)?;

    for node in nodes {
        // TODO: check if the link already exists.
        for (dest, _) in &node.links {
            const STMNT: &str = "SELECT source, dest FROM links WHERE source = ?1 AND dest = ?2;";
            let res = state.sqlite.query_one(STMNT, [&node.uuid, &dest], |row| {
                Ok(row.get::<usize, String>(0).unwrap())
            });
            println!("Link Result: {res:?}");
            if let Err(_) = res {
                let source = &node.uuid;
                rebuild::insert_link(&mut state.sqlite.connection(), &source, &dest)?;
                state.dynamic_state.updated_links.push(RoamLink {
                    from: source.clone().into(),
                    to: dest.clone().into(),
                });
            }
        }

        const STMNT: &str = "SELECT id FROM nodes WHERE id = ?1;";
        let res = state.sqlite.query_one(STMNT, [&node.uuid], |row| {
            Ok(row.get::<usize, String>(0).unwrap())
        });
        println!("Node result {:?}: {res:?}", node.title);
        if let Err(_) = res {
            node.insert_into(&mut state.sqlite.connection())?;
            state.dynamic_state.updated_nodes.push(node.into());
            state.dynamic_state.pending_reload = true;
        }
    }

    println!("Dynamic state: {:?}", state.dynamic_state);

    Ok(())
}

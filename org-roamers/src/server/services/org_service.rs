use orgize::Org;

use crate::server::types::{IncomingLink, OrgAsHTMLResponse, OutgoingLink, RoamID, RoamTitle};
use crate::server::AppState;
use crate::transform::export::HtmlExport;
use crate::transform::subtree::Subtree;

#[derive(Debug)]
pub enum Query {
    ByTitle(RoamTitle),
    ById(RoamID),
}

pub fn get_org_as_html(app_state: AppState, query: Query, scope: String) -> OrgAsHTMLResponse {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;

    // TODO: remove unwraps
    let (id, cache_entry) = match query {
        Query::ByTitle(ref title) => server_state
            .cache
            .get_by_name(&mut server_state.sqlite.connection(), title.title())
            .unwrap(),
        Query::ById(ref id) => (id.clone(), server_state.cache.retrieve(&id).unwrap()),
    };

    let contents = if scope == "file" {
        cache_entry.content().to_string()
    } else {
        Subtree::get(id.into(), cache_entry.content()).unwrap_or(cache_entry.content().to_string())
    };

    // Convert absolute path to relative path from org-roam directory
    let relative_file = cache_entry.path().to_string_lossy().into_owned();

    let mut handler = HtmlExport::new(&server_state.config.org_to_html, relative_file);
    Org::parse(contents).traverse(&mut handler);

    let (org, outgoing_links, latex_blocks) = handler.finish();

    tracing::info!(
        "Generated HTML length: {}, LaTeX blocks: {}, outgoing links: {}",
        org.len(),
        latex_blocks.len(),
        outgoing_links.len()
    );

    let outgoing_links = {
        outgoing_links
            .iter()
            .map(|bare| {
                const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?1";
                server_state.sqlite.query_one(STMNT, [bare], |row| {
                    Ok(OutgoingLink {
                        display: row.get::<usize, String>(1).unwrap().into(),
                        id: row.get::<usize, String>(0).unwrap().into(),
                    })
                })
            })
            .filter_map(|res| match res {
                Ok(link) => Some(link),
                Err(err) => {
                    tracing::error!("An error occurred: {err:?}");
                    None
                }
            })
            .collect()
    };

    let incoming_links = {
        let id = match query {
            Query::ByTitle(title) => {
                const STMNT: &str = "SELECT n.id FROM nodes n WHERE n.id = ?1";
                server_state
                    .sqlite
                    .query_one(STMNT, [title.title()], |row| {
                        Ok(RoamID::from(row.get::<usize, String>(0).unwrap()))
                    })
                    .unwrap()
            }
            Query::ById(id) => id,
        };

        const STMNT: &str = r#"
            SELECT n.id, n.title
            FROM links l
            JOIN nodes n ON l.source = n.id
            WHERE l.dest = ?1;
        "#;
        let mut stmnt = server_state.sqlite.connection().prepare(STMNT).unwrap();
        stmnt
            .query_map([id.id()], |row| {
                Ok(IncomingLink {
                    display: RoamTitle::from(row.get::<usize, String>(1).unwrap()),
                    id: RoamID::from(row.get::<usize, String>(0).unwrap()),
                })
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };

    OrgAsHTMLResponse {
        org,
        outgoing_links,
        incoming_links,
        latex_blocks,
    }
}

use std::path::{Path, PathBuf};

use orgize::Org;

use crate::server::types::{IncomingLink, OrgAsHTMLResponse, OutgoingLink, RoamID, RoamTitle};
use crate::server::AppState;
use crate::sqlite::helpers;
use crate::transform::export::HtmlExport;
use crate::transform::subtree::Subtree;
use crate::FileProcessingGuard;

#[derive(Debug)]
pub enum Query {
    ByTitle(RoamTitle),
    ById(RoamID),
}

pub fn get_org_as_html(app_state: AppState, query: Query, scope: String) -> OrgAsHTMLResponse {
    tracing::info!("Org request: query={:?}, scope={}", query, scope);

    let (file_path, id, db_result) = {
        let mut state = app_state.lock().unwrap();
        let (ref mut server_state, _) = *state;

        let [_title, id, file] =
            match helpers::get_all_nodes(server_state.sqlite.connection(), ["title", "id", "file"])
                .into_iter()
                .find(|[title, node, _]| match &query {
                    Query::ByTitle(name) => title.contains(name.title()),
                    Query::ById(id) => node.contains(id.id()),
                }) {
                Some(node) => {
                    tracing::info!("Found node: id={}, file={}", node[1], node[2]);
                    node
                }
                None => {
                    tracing::error!("Node not found for query: {:?}", query);
                    return OrgAsHTMLResponse::simple("Did not get node.");
                }
            };

        let file = file.replace('"', "");
        (
            PathBuf::from(&file),
            id,
            (
                file,
                server_state.org_roam_db_path.clone(),
                server_state.html_export_settings.clone(),
            ),
        )
    };

    // Create file processing guard to prevent watcher conflicts
    let _guard = match FileProcessingGuard::new(app_state.clone(), file_path.clone()) {
        Ok(guard) => guard,
        Err(_) => {
            return OrgAsHTMLResponse::simple("Could not acquire file processing lock");
        }
    };

    let (file, org_roam_db_path, html_export_settings) = db_result;

    let contents = match std::fs::read_to_string(&file) {
        Ok(f) => f,
        Err(err) => {
            return OrgAsHTMLResponse::simple(format!("Could not get file contents: {err}"))
        }
    };

    let contents = if scope == "file" {
        contents
    } else {
        Subtree::get(id.into(), contents.as_str()).unwrap_or(contents)
    };

    // Convert absolute path to relative path from org-roam directory
    let relative_file = PathBuf::from(&file)
        .strip_prefix(&org_roam_db_path)
        .unwrap_or(Path::new(&file))
        .to_string_lossy()
        .to_string();

    let mut handler = HtmlExport::new(&html_export_settings, relative_file);
    Org::parse(contents).traverse(&mut handler);

    let (org, outgoing_links, latex_blocks) = handler.finish();

    tracing::info!(
        "Generated HTML length: {}, LaTeX blocks: {}, outgoing links: {}",
        org.len(),
        latex_blocks.len(),
        outgoing_links.len()
    );

    let outgoing_links = {
        let mut state = app_state.lock().unwrap();
        let (ref mut server_state, _) = *state;

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
        let mut state = app_state.lock().unwrap();
        let (ref mut server_state, _) = *state;

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

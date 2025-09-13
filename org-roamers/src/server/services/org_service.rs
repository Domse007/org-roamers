use std::path::{Path, PathBuf};

use orgize::Org;

use crate::server::types::{OrgAsHTMLResponse, OutgoingLink, RoamID, RoamTitle};
use crate::sqlite::helpers;
use crate::transform::export::HtmlExport;
use crate::transform::subtree::Subtree;
use crate::ServerState;

pub enum Query {
    ByTitle(RoamTitle),
    ById(RoamID),
}

pub fn get_org_as_html(db: &mut ServerState, query: Query, scope: String) -> OrgAsHTMLResponse {
    let [_title, id, file] =
        match helpers::get_all_nodes(db.sqlite.connection(), ["title", "id", "file"])
            .into_iter()
            .find(|[title, node, _]| match &query {
                Query::ByTitle(name) => title.contains(name.title()),
                Query::ById(id) => node.contains(id.id()),
            }) {
            Some(node) => node,
            None => return OrgAsHTMLResponse::simple("Did not get node."),
        };

    let file = file.replace('"', "");

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
        .strip_prefix(&db.org_roam_db_path)
        .unwrap_or(Path::new(&file))
        .to_string_lossy()
        .to_string();

    let mut handler = HtmlExport::new(&db.html_export_settings, relative_file);
    Org::parse(contents).traverse(&mut handler);

    let (org, outgoing_links) = handler.finish();

    let links = outgoing_links
        .iter()
        .map(|bare| {
            const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?1";
            db.sqlite.query_one(STMNT, [bare], |row| {
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
        .collect();

    OrgAsHTMLResponse { org, links }
}
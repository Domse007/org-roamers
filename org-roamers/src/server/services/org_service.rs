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

pub async fn get_org_as_html(
    app_state: AppState,
    query: Query,
    scope: String,
) -> OrgAsHTMLResponse {
    // Get data from cache and extract needed values
    let (id, content, path, config, sqlite) = {
        let sqlite_pool = app_state.lock().unwrap().sqlite.clone();

        let (id, content, path) = match &query {
            Query::ByTitle(title) => {
                let stmnt = r#"
                    SELECT id FROM nodes
                    WHERE title = ?;
                "#;
                let (id_str,): (String,) = sqlx::query_as(stmnt)
                    .bind(title.title())
                    .fetch_one(&sqlite_pool)
                    .await
                    .unwrap();

                let state = app_state.lock().unwrap();
                let id: RoamID = id_str.into();
                let cache_entry = state.cache.retrieve(&id).unwrap();
                (
                    id,
                    cache_entry.content().to_string(),
                    cache_entry.path().to_path_buf(),
                )
            }
            Query::ById(id) => {
                let state = app_state.lock().unwrap();
                let cache_entry = state.cache.retrieve(&id).unwrap();
                (
                    id.clone(),
                    cache_entry.content().to_string(),
                    cache_entry.path().to_path_buf(),
                )
            }
        };

        let state = app_state.lock().unwrap();
        let config = state.config.clone();
        (id, content, path, config, sqlite_pool)
    };

    let contents = if scope == "file" {
        content.clone()
    } else {
        Subtree::get(id.clone().into(), &content).unwrap_or(content.clone())
    };

    // Convert absolute path to relative path from org-roam directory
    let relative_file = path.to_string_lossy().into_owned();

    let mut handler = HtmlExport::new(&config.org_to_html, relative_file);
    Org::parse(contents).traverse(&mut handler);

    let (org, org_outgoing_links, latex_blocks) = handler.finish();

    tracing::info!(
        "Generated HTML length: {}, LaTeX blocks: {}, outgoing links: {}",
        org.len(),
        latex_blocks.len(),
        org_outgoing_links.len()
    );

    let mut outgoing_links = vec![];
    for link_id in org_outgoing_links {
        const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?";
        let res = sqlx::query_as::<_, (String, String)>(STMNT)
            .bind(&link_id)
            .fetch_one(&sqlite)
            .await;
        match res {
            Ok((id, display)) => {
                outgoing_links.push(OutgoingLink {
                    display: RoamTitle::from(display),
                    id: RoamID::from(id),
                });
            }
            Err(err) => {
                tracing::error!("Failed to fetch outgoing link for id {}: {}", link_id, err);
            }
        }
    }

    let final_id: RoamID = match query {
        Query::ByTitle(title) => {
            let (id_str,): (String,) = sqlx::query_as("SELECT n.id FROM nodes n WHERE n.title = ?")
                .bind(title.title())
                .fetch_one(&sqlite)
                .await
                .unwrap();
            RoamID::from(id_str)
        }
        Query::ById(id) => id,
    };

    const STMNT: &str = r#"
            SELECT n.id, n.title
            FROM links l
            JOIN nodes n ON l.source = n.id
            WHERE l.dest = ?
        "#;

    let incoming_links = sqlx::query_as::<_, (String, String)>(STMNT)
        .bind(final_id.id())
        .fetch_all(&sqlite)
        .await
        .map(|list| {
            list.into_iter()
                .map(|(id, disp): (String, String)| IncomingLink {
                    display: RoamTitle::from(disp),
                    id: RoamID::from(id),
                })
                .collect()
        })
        .unwrap();

    OrgAsHTMLResponse {
        org,
        outgoing_links,
        incoming_links,
        latex_blocks,
    }
}

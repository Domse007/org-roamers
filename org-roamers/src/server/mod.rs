use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use axum::{
    extract::{Query as AxumQuery, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use tower_http::cors::CorsLayer;

use data::DataLoader;
use emacs::route_emacs_traffic;
use orgize::Org;
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;

use crate::api::types::GraphData;
use crate::api::types::OrgAsHTMLResponse;
use crate::api::types::OutgoingLink;
use crate::api::types::RoamID;
use crate::api::types::RoamLink;
use crate::api::types::RoamNode;
use crate::api::types::RoamTitle;
use crate::api::types::SearchResponse;
use crate::api::types::SearchResponseProvider;
use crate::api::types::ServerStatus;
use crate::api::APICalls;
use crate::api::APICallsInternal;
use crate::diff;
use crate::latex;
use crate::search::Search;
use crate::server::emacs::EmacsRequest;
use crate::sqlite::helpers;
use crate::sqlite::olp;
use crate::transform::export::HtmlExport;
use crate::transform::subtree::Subtree;
use crate::transform::title::TitleSanitizer;
use crate::watcher;
use crate::ServerState;

pub mod data;
pub mod emacs;

type AppState = Arc<Mutex<(ServerState, APICallsInternal, Arc<Mutex<bool>>)>>;

pub struct ServerRuntime {
    handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>,
    sender: Sender<()>,
}

impl ServerRuntime {
    pub fn graceful_shutdown(self) -> Result<(), Box<dyn Any + Send>> {
        self.sender.send(()).unwrap();
        match self.handle.join() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(Box::new(e) as Box<dyn Any + Send>),
            Err(e) => Err(e),
        }
    }
}

pub fn start_server(
    url: String,
    calls: APICalls,
    state: ServerState,
) -> Result<ServerRuntime, Box<dyn Error>> {
    tracing::info!("Using server configuration: {:?}", state.static_conf);

    tracing::info!(
        "Using HTML settings: {}",
        serde_json::to_string(&state.html_export_settings).unwrap()
    );

    let calls: APICallsInternal = calls.into();

    let org_roam_db_path = state.org_roam_db_path.clone();
    let use_fs_watcher = state.static_conf.fs_watcher;

    let mut changes_flag = Arc::new(Mutex::new(false));
    if use_fs_watcher {
        tracing::info!("Starting fs watcher.");
        let (_watcher, _changes_flag) = watcher::watcher(org_roam_db_path.clone())?;
        changes_flag = _changes_flag;
        // For now, disable file watcher integration to avoid state cloning issues
        // This will be properly implemented in a future update
        tracing::warn!("File watcher integration temporarily disabled in Axum version");
    }

    let app_state = Arc::new(Mutex::new((state, calls, changes_flag.clone())));

    let app = Router::new()
        .route("/", get(default_route))
        .route("/org", get(get_org_as_html_handler))
        .route("/search", get(search_handler))
        .route("/graph", get(get_graph_data_handler))
        .route("/latex", get(get_latex_svg_handler))
        .route("/status", get(get_status_handler))
        .route("/emacs", post(emacs_handler))
        .route("/assets", get(serve_assets_handler))
        .fallback(fallback_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(&url).await?;
            tracing::info!("Server listening on {}", url);

            let server = axum::serve(listener, app);

            // Set up graceful shutdown
            tokio::select! {
                result = server => {
                    if let Err(err) = result {
                        tracing::error!("Server error: {}", err);
                        return Err(Box::new(err) as Box<dyn Error + Send + Sync>);
                    }
                },
                _ = async {
                    while shutdown_rx.try_recv().is_err() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                } => {
                    tracing::info!("Shutdown signal received");
                }
            }
            Ok(())
        })
    });

    Ok(ServerRuntime {
        handle,
        sender: shutdown_tx,
    })
}

async fn default_route(State(app_state): State<AppState>) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;
    let conf = server_state.static_conf.root.to_string();
    calls.default_route(server_state, conf, None)
}

async fn get_org_as_html_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;

    let scope = params
        .get("scope")
        .cloned()
        .unwrap_or_else(|| "file".to_string());

    let query = match params.get("id") {
        Some(id) => Query::ById(id.clone().into()),
        None => match params.get("title") {
            Some(title) => Query::ByTitle(title.clone().into()),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    calls
        .get_org_as_html(server_state, query, scope)
        .into_response()
}

async fn search_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;

    match params.get("q") {
        Some(query) => calls
            .serve_search_results(server_state, query.clone())
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_graph_data_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;
    calls.get_graph_data(server_state)
}

async fn get_latex_svg_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;

    match (params.get("tex"), params.get("color"), params.get("id")) {
        (Some(tex), Some(color), Some(id)) => {
            calls.serve_latex_svg(server_state, tex.clone(), color.clone(), id.clone())
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_status_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, ref changes_flag) = *state;
    calls.get_status_data(server_state, changes_flag.clone())
}

async fn emacs_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    tracing::debug!("Emacs request with params: {:?}", params);

    match route_emacs_traffic(params) {
        Ok(req) => {
            let mut state = app_state.lock().unwrap();
            let (ref mut server_state, _, _) = *state;

            match req {
                EmacsRequest::BufferOpened(id) => {
                    server_state.dynamic_state.update_working_id(id.into());
                }
                EmacsRequest::BufferModified(file) => {
                    if let Err(err) = diff::diff(server_state, file) {
                        tracing::error!("An error occurred while updating db: {err}");
                    }
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}

async fn serve_assets_handler(AxumQuery(params): AxumQuery<HashMap<String, String>>) -> Response {
    match params.get("file") {
        Some(path) => serve_assets(path.clone()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn fallback_handler(uri: axum::http::Uri, State(app_state): State<AppState>) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, ref mut calls, _) = *state;
    let conf = server_state.static_conf.root.to_string();
    calls.default_route(server_state, conf, Some(uri.path().to_string()))
}

pub fn default_route_content(_db: &mut ServerState, root: String, url: Option<String>) -> Response {
    let root = PathBuf::from(root);

    let rel_path = match url {
        Some(url) => PathBuf::from(url.strip_prefix("/").unwrap_or(&url)),
        None => PathBuf::from("index.html"),
    };

    let mime = match rel_path.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "html" => "text/html",
            "js" => "text/javascript",
            "css" => "text/css",
            "ico" => "image/x-icon",
            _ => {
                tracing::error!(
                    "Unsupported file extension: {:?} ({:?})",
                    rel_path.extension(),
                    rel_path
                );
                return StatusCode::NOT_FOUND.into_response();
            }
        },
        _ => {
            tracing::error!("No file extension provided.");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let asset_loader = data::get_loader(root);

    let bytes = match asset_loader.load(&rel_path) {
        Some(bytes) => {
            tracing::info!("Serving file {rel_path:?}");
            bytes
        }
        None => {
            tracing::error!("File not found: {rel_path:?}");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.parse().unwrap());

    (StatusCode::OK, headers, bytes).into_response()
}

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

    // FIXME: This is VERY BAD!! file is an absolute path, but it should be
    //        relative to the root of the org-roam dir.
    let mut handler = HtmlExport::new(&db.html_export_settings, file);
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

pub fn search(db: &mut ServerState, query: String) -> SearchResponse {
    let search = Search::new(query.as_str());
    let res = search.search(db.sqlite.connection());

    let nodes = match res {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("An error occurred while providing search: {err}");
            return SearchResponse { providers: vec![] };
        }
    };

    SearchResponse {
        providers: vec![SearchResponseProvider {
            source: "sqlite".to_string(),
            results: nodes,
        }],
    }
}

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

pub fn get_latex_svg(db: &mut ServerState, tex: String, color: String, id: String) -> Response {
    let node = helpers::get_all_nodes(db.sqlite.connection(), ["file", "id"])
        .into_iter()
        .find(|[_, c_id]| c_id.contains(&id));

    let svg = match node {
        Some([file, _]) => {
            let file = file.replace('"', "");
            latex::get_image_with_ctx(tex, color, file)
        }
        None => latex::get_image(tex, color, vec![]),
    };

    match svg {
        Ok(svg) => {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", "image/svg+xml".parse().unwrap());
            (StatusCode::OK, headers, svg).into_response()
        }
        Err(err) => {
            let error_msg = format!("Could not generate svg: {:#?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}

pub fn get_status_data(state: &mut ServerState, changes_flag: Arc<Mutex<bool>>) -> ServerStatus {
    let mut changes = changes_flag.lock().unwrap();

    let mut updated_nodes = state
        .dynamic_state
        .updated_nodes
        .drain(..)
        .collect::<Vec<_>>();
    updated_nodes.sort();
    updated_nodes.dedup();

    let mut updated_links = state
        .dynamic_state
        .updated_links
        .drain(..)
        .collect::<Vec<_>>();
    updated_links.sort();
    updated_links.dedup();

    let status = ServerStatus {
        visited_node: state.dynamic_state.get_working_id().cloned(),
        pending_changes: *changes || state.dynamic_state.pending_reload,
        updated_nodes,
        updated_links,
    };
    *changes = false;
    status
}

pub fn serve_assets(file: String) -> Response {
    let file = PathBuf::from(file);

    let mime = match file.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            _ => return StatusCode::NOT_FOUND.into_response(),
        },
        _ => {
            tracing::error!("No file extension provided.");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let mut buffer = vec![];
    let mut source_file = match File::open(file) {
        Ok(file) => file,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    if source_file.read_to_end(&mut buffer).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.parse().unwrap());

    (StatusCode::OK, headers, buffer).into_response()
}

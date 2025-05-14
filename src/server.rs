use std::any::Any;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Instant;

use orgize::Org;
use rouille::{router, Response, Server};
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use serde::Deserialize;
use serde::Serialize;

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
use crate::export::HtmlExport;
use crate::latex;
use crate::search::Search;
use crate::sqlite::helpers;
use crate::sqlite::olp;
use crate::subtree::Subtree;
use crate::transform::title::TitleSanitizer;
use crate::watcher;
use crate::ServerState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfiguration {
    /// Root path to the website files. e.g. .js / .html / .css
    pub root: String,
}

pub struct ServerRuntime {
    handle: JoinHandle<()>,
    sender: Sender<()>,
}

impl ServerRuntime {
    pub fn graceful_shutdown(self) -> Result<(), Box<dyn Any + Send>> {
        self.sender.send(()).unwrap();
        self.handle.join()
    }
}

pub fn start_server(
    url: String,
    conf: ServerConfiguration,
    calls: APICalls,
    _global: ServerState,
    // TODO: move to some struct.
    sqlite_path: PathBuf,
) -> Result<ServerRuntime, Box<dyn Error>> {
    tracing::info!("Using server configuration: {conf:?}");
    let conf: &'static ServerConfiguration = Box::leak(Box::new(conf));

    tracing::info!(
        "Using HTML settings: {}",
        serde_json::to_string(&_global.html_export_settings).unwrap()
    );

    let lock = Arc::new(Mutex::new(_global));

    let (watcher, changes_flag) = watcher::watcher(sqlite_path.clone())?;
    watcher::default_watcher_runtime(lock.clone(), watcher, sqlite_path);

    let server = Server::new(url, move |request| {
        let mut global = lock.lock().unwrap();
        router!(request,
            (GET) (/)  => {
                (calls.default_route)(&mut global, conf.root.to_string(), None)
            },
            (GET) (/org) => {
                let scope = request.get_param("scope")
                    .unwrap_or_else(|| "file".to_string());
                let query = match request.get_param("id") {
                    Some(id) => Query::ById(id.into()),
                    None => {
                        match request.get_param("title") {
                            Some(title) => Query::ByTitle(title.into()),
                            None => return Response::empty_404(),
                        }
                    }
                };
                (calls.get_org_as_html)(&mut global, query, scope).into()
            },
            (GET) (/search) => {
                match request.get_param("q") {
                    Some(query) => (calls.serve_search_results)(&mut global, query),
                    None => Response::empty_404(),
                }
            },
            (GET) (/graph) => {
                get_graph_data(&mut global)
            },
            (GET) (/latex) => {
                match request.get_param("tex") {
                    Some(tex) => (calls.serve_latex_svg)(
                        &mut global,
                        tex,
                        request.get_param("color").unwrap(),
                        request.get_param("id").unwrap(),
                    ),
                    None => Response::empty_404(),
                }
            },
            (GET) (/status) => {
                get_status_data(&mut global, changes_flag.clone()).into()
            },
            _ => (calls.default_route)(&mut global, conf.root.to_string(), Some(request.url()))
        )
    })
    .unwrap();

    let (handle, sender) = server.stoppable();

    Ok(ServerRuntime { handle, sender })
}

pub fn default_route_content(_db: &mut ServerState, root: String, url: Option<String>) -> Response {
    let mut path = PathBuf::from(root);

    match url {
        Some(url) => path.push(url.strip_prefix("/").unwrap()),
        None => path.push("index.html"),
    }

    let mime = match path.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "html" => "text/html",
            "js" => "text/javascript",
            "css" => "text/css",
            "ico" => "image/x-icon",
            _ => {
                tracing::error!(
                    "Unsupported file extension: {:?} ({:?})",
                    path.extension(),
                    path
                );
                return Response::empty_404();
            }
        },
        _ => {
            tracing::error!("No file extension provided.");
            return Response::empty_404();
        }
    };

    let file = match File::open(&path) {
        Ok(file) => {
            tracing::info!("Serving file {path:?}");
            file
        }
        Err(_) => {
            tracing::error!("File not found: {path:?}");
            return Response::empty_404();
        }
    };

    Response::from_file(mime, file)
}

pub enum Query {
    ByTitle(RoamTitle),
    ById(RoamID),
}

pub fn get_org_as_html(db: &mut ServerState, query: Query, scope: String) -> OrgAsHTMLResponse {
    let before = Instant::now();

    let [_title, id, file] =
        match helpers::get_all_nodes(db.sqlite.connection(), ["title", "id", "file"])
            .into_iter()
            .filter(|[title, node, _]| match &query {
                Query::ByTitle(name) => title.contains(name.title()),
                Query::ById(id) => node.contains(id.id()),
            })
            .next()
        {
            Some(node) => node,
            None => return OrgAsHTMLResponse::simple("Did not get node."),
        };

    let contents = match std::fs::read_to_string(file.replace('"', "")) {
        Ok(f) => f,
        Err(_) => return OrgAsHTMLResponse::simple("Could not get file contents."),
    };

    let contents = if scope == "file" {
        contents
    } else {
        Subtree::new(id.into(), contents.as_str()).unwrap_or(contents)
    };

    let mut handler = HtmlExport::new(&db.html_export_settings);
    Org::parse(contents).traverse(&mut handler);

    let (org, outgoing_links) = handler.finish();

    let links = outgoing_links
        .iter()
        .map(|bare| {
            let bare = format!("\"{}\"", bare);
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
                tracing::error!("An error occured: {err:?}");
                None
            }
        })
        .collect();

    let delta = Instant::now() - before;
    tracing::info!("Translating org to HTML took {}ms.", delta.as_millis());

    OrgAsHTMLResponse { org, links }
}

pub fn search(db: &mut ServerState, query: String) -> Response {
    let search = Search::new(query.as_str());
    let res = search.search(db.sqlite.connection());

    let nodes = match res {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("An error occured while prividing search: {err}");
            return SearchResponse { providers: vec![] }.into();
        }
    };

    SearchResponse {
        providers: vec![SearchResponseProvider {
            source: "sqlite".to_string(),
            results: nodes,
        }],
    }
    .into()
}

pub fn get_graph_data(mut db: &mut ServerState) -> Response {
    let before = Instant::now();

    let olp = |s: String, db: &mut ServerState| -> String {
        (!s.is_empty())
            .then(|| {
                olp::parse_olp(s[1..s.len() - 1].to_string())
                    .unwrap_or_default()
                    .pop()
                    .unwrap_or_default()
            })
            .map(|parent| {
                let stmnt = "SELECT title, id FROM nodes WHERE title = ?1;";
                // TODO: fix at some point
                let parent = format!("\"{}\"", parent);
                db.sqlite.query_one(stmnt, [parent], |row| {
                    Ok(row.get::<usize, String>(1).unwrap())
                })
            })
            .unwrap_or(Ok(String::new()))
            .unwrap_or_default()
    };

    let title_sanitizer = |title: &str| {
        let sanitier = TitleSanitizer::new();
        sanitier.process(title)
    };

    let mut nodes = helpers::get_all_nodes(db.sqlite.connection(), ["id", "title", "actual_olp"])
        .into_iter()
        .map(|e| RoamNode {
            title: title_sanitizer(&e[1]).into(),
            id: e[0].to_string().into(),
            parent: olp(e[2].to_string(), &mut db).into(),
            num_links: 0,
        })
        .collect::<Vec<RoamNode>>();

    const STMNT: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = '\"id\"'\n",
        "AND (dest = ?1 OR source = ?1)"
    );

    let mut stmnt = db.sqlite.connection().prepare(STMNT).unwrap();
    for node in &mut nodes {
        let num = stmnt
            .query([node.id.with_quotes(1)])
            .unwrap()
            .count()
            .unwrap();
        node.num_links = num;
    }

    drop(stmnt);

    const ALL_LINKS: &str = concat!(
        "SELECT source, dest, type\n",
        "FROM links\n",
        "WHERE type = '\"id\"';"
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

    let delta = Instant::now() - before;
    tracing::info!("Building graph response took {}ms.", delta.as_millis());

    GraphData { nodes, links }.into()
}

pub fn get_latex_svg(db: &mut ServerState, tex: String, color: String, id: String) -> Response {
    let node = helpers::get_all_nodes(db.sqlite.connection(), ["file", "id"])
        .into_iter()
        .filter(|[_, c_id]| c_id.contains(&id))
        .next();

    let svg = match node {
        Some([file, _]) => {
            let file = file.replace('"', "");
            latex::get_image_with_ctx(tex, color, file)
        }
        None => latex::get_image(tex, color, vec![]),
    };

    match svg {
        Ok(svg) => Response::svg(svg),
        Err(err) => Response::text(format!("Could not generate svg: {:#?}", err)),
    }
}

pub fn get_status_data(_db: &mut ServerState, changes_flag: Arc<Mutex<bool>>) -> ServerStatus {
    let mut changes = changes_flag.lock().unwrap();
    let status = ServerStatus {
        pending_changes: *changes,
    };
    *changes = false;
    status
}

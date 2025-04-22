use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread::JoinHandle;

use orgize::Org;
use rouille::{router, Response, Server};
use serde::Serialize;

use crate::api::APICalls;
use crate::export::HtmlExport;
use crate::get_nodes_internal;
use crate::latex;
use crate::sqlite::SqliteConnection;
use crate::GetNodesResultWrapper;
use crate::Global;
use crate::DB;

pub static WEBSERVER: Mutex<Option<(JoinHandle<()>, Sender<()>)>> = Mutex::new(None);

pub fn start_server(url: String, root: String, calls: APICalls) -> Result<(), Box<dyn Error>> {
    if WEBSERVER.lock().unwrap().is_some() {
        return Err("Server already running.".into());
    }

    let root: &'static str = Box::leak(Box::new(root));

    let server = Server::new(url, move |request| {
        router!(request,
            (GET) (/)  => {
                (calls.default_route)(root.to_string(), None)
            },
            (GET) (/org) => {
                match request.get_param("title") {
                    Some(title) => (calls.get_org_as_html)(title),
                    None => Response::empty_404(),
                }
            },
            (GET) (/search) => {
                match request.get_param("q") {
                    Some(query) => (calls.serve_search_results)(query),
                    None => Response::empty_404(),
                }
            },
            (GET) (/graph) => {
                get_graph_data()
            },
            (GET) (/latex) => {
                match request.get_param("tex") {
                    Some(tex) => (calls.serve_latex_svg)(
                        tex,
                        request.get_param("color").unwrap(),
                        request.get_param("title").unwrap(),
                    ),
                    None => Response::empty_404(),
                }
            },
            _ => (calls.default_route)(root.to_string(), Some(request.url()))
        )
    })
    .unwrap();

    let ctx = server.stoppable();

    let mut ws = WEBSERVER.lock().unwrap();
    *ws = Some(ctx);

    Ok(())
}

fn stop_server() -> Result<(), Box<dyn Error>> {
    if WEBSERVER.lock().unwrap().is_none() {
        return Err("No server is running.".into());
    }

    let mut server = WEBSERVER.lock().unwrap();

    let (handle, sender) = server.take().unwrap();

    sender.send(()).unwrap();

    handle.join().unwrap();

    Ok(())
}

pub fn default_route_content(root: String, url: Option<String>) -> Response {
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
            _ => return Response::empty_404(),
        },
        _ => return Response::empty_404(),
    };

    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Response::empty_404(),
    };

    Response::from_file(mime, file)
}

pub fn get_org_as_html(name: String) -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let [_title, _id, file] = match db
        .sqlite
        .get_all_nodes(["title", "id", "file"])
        .into_iter()
        .filter(|[title, node, _]| title.contains(&name) || node.contains(&name))
        .next()
    {
        Some(node) => node,
        None => return Response::text("Did not get node."),
    };

    // FIXME: This does not narrow to the node, but only to the file.
    let contents = match std::fs::read_to_string(file.replace('"', "")) {
        Ok(f) => f,
        Err(_) => return Response::text("Could not get file contents."),
    };

    let mut handler = HtmlExport::default();
    Org::parse(contents).traverse(&mut handler);

    Response::text(handler.finish())
}

#[derive(Serialize)]
struct SearchResult {
    tantivy: GetNodesResultWrapper,
    sqlite: Vec<String>,
}

pub fn search(query: String) -> Response {
    let logger = crate::logger::StdOutLogger;
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let nodes = db
        .sqlite
        .get_all_nodes(["title"])
        .into_iter()
        .filter(|[file]| file.contains(&query))
        .take(10)
        .map(|e| e[0][1..e[0].len() - 1].to_string())
        .collect();

    let tan_result = match get_nodes_internal(db, logger, query, 10) {
        Ok(result) => result,
        Err(_) => return Response::empty_404(),
    };

    let result = SearchResult {
        tantivy: tan_result,
        sqlite: nodes,
    };

    let json = match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(_) => return Response::empty_404(),
    };

    Response::json(&json)
}

#[derive(Serialize)]
pub struct GraphData {
    /// The tuple is (id, title, parent)
    nodes: Vec<(String, String, String)>,
    edges: Vec<(String, String)>,
}

pub fn get_graph_data() -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let mut db = db.as_mut().unwrap();

    let olp = |s: String, db: &mut Global| {
        (!s.is_empty())
            .then(|| {
                SqliteConnection::parse_olp(s)
                    .unwrap_or_default()
                    .pop()
                    .unwrap_or_default()
            })
            .map(|parent| db.sqlite.get_id_by_title(parent.as_str()))
            .unwrap_or_default()
            .unwrap_or_default()
    };

    let nodes = db
        .sqlite
        .get_all_nodes(["id", "title", "olp"])
        .into_iter()
        .map(|e| {
            (
                e[0].to_string(),
                e[1].to_string(),
                olp(e[2].to_string(), &mut db),
            )
        })
        .collect::<Vec<(String, String, String)>>();

    let mut edges = db.sqlite.get_all_links();

    for node in &nodes {
        if let Some(parent_id) = db.sqlite.get_parent_for_id(node.0.as_str()) {
            edges.push((parent_id, node.0.to_string()));
        }
    }

    Response::json(&serde_json::to_string(&GraphData { nodes, edges }).unwrap())
}

pub fn get_latex_svg(tex: String, color: String, title: String) -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let node = db
        .sqlite
        .get_all_nodes(["file", "title"])
        .into_iter()
        .filter(|[_, c_title]| c_title.contains(&title))
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

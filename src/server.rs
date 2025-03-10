use std::fs::read_to_string;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread::JoinHandle;

use emacs::defun;
use emacs::Result;
use orgize::Org;
use rouille::{router, Response, Server};
use serde::Serialize;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::TantivyDocument;

use crate::get_nodes_internal;
use crate::latex;
use crate::DB;

pub static WEBSERVER: Mutex<Option<(JoinHandle<()>, Sender<()>)>> = Mutex::new(None);

#[defun]
pub fn start_server(url: String, root: String) -> Result<()> {
    if WEBSERVER.lock().unwrap().is_some() {
        return Err(emacs::Error::msg("Server already running."));
    }

    let root: &'static str = Box::leak(Box::new(root));

    let server = Server::new(url, move |request| {
        router!(request,
            (GET) (/)  => {
                default_route_content(root, None)
            },
            (GET) (/org) => {
                match request.get_param("title") {
                    Some(title) => get_org_as_html(title),
                    None => Response::empty_404(),
                }
            },
            (GET) (/search) => {
                match request.get_param("q") {
                    Some(query) => search(query),
                    None => Response::empty_404(),
                }
            },
            (GET) (/graph) => {
                get_graph_data()
            },
            (GET) (/latex) => {
                match request.get_param("tex") {
                    Some(tex) => get_latex_svg(
                        tex,
                        request.get_param("color").unwrap(),
                        request.get_param("title").unwrap(),
                    ),
                    None => Response::empty_404(),
                }
            },
            _ => default_route_content(root, Some(request.url()))
        )
    })
    .unwrap();

    let ctx = server.stoppable();

    let mut ws = WEBSERVER.lock().unwrap();
    *ws = Some(ctx);

    Ok(())
}

#[defun]
fn stop_server() -> Result<()> {
    if WEBSERVER.lock().unwrap().is_none() {
        return Err(emacs::Error::msg("No server is running."));
    }

    let mut server = WEBSERVER.lock().unwrap();

    let (handle, sender) = server.take().unwrap();

    sender.send(()).unwrap();

    handle.join().unwrap();

    Ok(())
}

fn default_route_content(root: &str, url: Option<String>) -> Response {
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

fn get_org_as_html(name: String) -> Response {
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

    let mut html = Org::parse(contents).to_html();

    Response::text(html)
}

fn search(query: String) -> Response {
    let logger = crate::logger::StdOutLogger;
    let result = match get_nodes_internal(logger, query, 10) {
        Ok(result) => result,
        Err(_) => return Response::empty_404(),
    };
    let json = match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(_) => return Response::empty_404(),
    };

    Response::json(&json)
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<(String, String)>,
    edges: Vec<(String, String)>,
}

fn get_graph_data() -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let nodes = db
        .sqlite
        .get_all_nodes(["id", "title"])
        .into_iter()
        .map(|e| (e[0].to_string(), e[1].to_string()))
        .collect::<Vec<(String, String)>>();

    let edges = db.sqlite.get_all_links();

    Response::json(&serde_json::to_string(&GraphData { nodes, edges }).unwrap())
}

fn get_latex_svg(tex: String, color: String, title: String) -> Response {
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

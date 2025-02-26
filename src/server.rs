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
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::TantivyDocument;

use crate::get_nodes_internal;
use crate::DB;

static WEBSERVER: Mutex<Option<(JoinHandle<()>, Sender<()>)>> = Mutex::new(None);

#[defun]
fn start_server(url: String, root: String) -> Result<()> {
    if WEBSERVER.lock().unwrap().is_some() {
        return Err(emacs::Error::msg("Server already running."));
    }

    let root: &'static str = Box::leak(Box::new(root));

    let server = Server::new(url, move |request| {
        router!(request,
            (GET) (/)  => {
                default_route_content(root)
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
            _ => Response::empty_404()
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

fn default_route_content(root: &str) -> Response {
    let mut path = PathBuf::from(root);
    path.push("index.html");
    
    let mut content = String::new();

    let mut file = match File::open(path.as_path()) {
        Ok(file) => file,
        Err(_) => return Response::empty_404(),
    };

    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(_) => return Response::empty_404(),
    }

    Response::html(content)
}

fn get_org_as_html(name: String) -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let searcher = db.index.reader().unwrap().searcher();
    let index = &db.index;
    let title_field = db.schema.get_field("title").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_field]);

    let query = match query_parser.parse_query(&format!("title:{name}")) {
        Ok(query) => query,
        Err(_) => return Response::empty_404(),
    };

    let res = match searcher.search(&query, &TopDocs::with_limit(1)) {
        Ok(res) => res,
        Err(_) => return Response::empty_404(),
    };

    let (_score, doc_address) = match res.into_iter().next() {
        Some(next) => next,
        None => return Response::empty_404(),
    };

    let retrieved_doc: TantivyDocument = match searcher.doc(doc_address) {
        Ok(doc) => doc,
        Err(_) => return Response::empty_404(),
    };

    let body_field = db.schema.get_field("body").unwrap();

    let body = retrieved_doc
        .get_first(body_field)
        .unwrap()
        .as_str()
        .unwrap();

    let mut html = Vec::new();

    Org::parse(body).write_html(&mut html).unwrap();

    Response::html(String::from_utf8(html).unwrap())
}

fn search(query: String) -> Response {
    let logger = crate::logger::StdOutLogger;
    let result = match get_nodes_internal(logger, query, 10) {
        Ok(result )=> result,
        Err(_) => return Response::empty_404(),
    };
    let json = match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(_) => return Response::empty_404(),
    };
    
    Response::json(&json)
}

// struct GraphData {
//     nodes: Vec<String>,
//     edges: Vec<(String, String)>,
// }

// fn get_graph_data() -> Response {
//     let db = &DB;
//     let mut db = db.get_mut().unwrap();
//     let db = db.as_mut().unwrap();

//     Response::empty_404()
// }

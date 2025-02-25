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

use crate::DB;

static WEBSERVER: Mutex<Option<(JoinHandle<()>, Sender<()>)>> = Mutex::new(None);

#[defun]
fn start_server(url: String) -> Result<()> {
    if WEBSERVER.lock().unwrap().is_some() {
        return Err(emacs::Error::msg("Server already running."));
    }

    let server = Server::new(url, move |request| {
        router!(request,
            (GET) (/)  => {
                Response::html("<body><h1>Hello, World!</h1></body>")
            },
            (GET) (/org) => {
                match request.get_param("title") {
                    Some(title) => get_org_as_html(title),
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

fn get_org_as_html(name: String) -> Response {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let searcher = db.index.reader().unwrap().searcher();
    let index = &db.index;
    let title_field = db.schema.get_field("title").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_field]);

    let query = query_parser.parse_query(&format!("title:{name}")).unwrap();

    let (_score, doc_address) = searcher
        .search(&query, &TopDocs::with_limit(1))
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();

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

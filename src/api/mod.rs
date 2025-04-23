pub mod types;

use rouille::Response;

use crate::ServerState;

pub struct APICalls {
    pub default_route: fn(&mut ServerState, String, Option<String>) -> Response,
    pub get_graph_data: fn(&mut ServerState) -> Response,
    pub get_org_as_html: fn(&mut ServerState, String) -> Response,
    pub serve_search_results: fn(&mut ServerState, String) -> Response,
    pub serve_latex_svg: fn(&mut ServerState, String, String, String) -> Response,
}

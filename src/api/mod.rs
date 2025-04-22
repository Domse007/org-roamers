pub mod types;

use rouille::Response;

use crate::Global;

pub struct APICalls {
    pub default_route: fn(&mut Global, String, Option<String>) -> Response,
    pub get_graph_data: fn(&mut Global) -> Response,
    pub get_org_as_html: fn(&mut Global, String) -> Response,
    pub serve_search_results: fn(&mut Global, String) -> Response,
    pub serve_latex_svg: fn(&mut Global, String, String, String) -> Response,
}

pub mod types;

use rouille::Response;

pub struct APICalls {
    pub default_route: fn(String, Option<String>) -> Response,
    pub get_graph_data: fn() -> Response,
    pub get_org_as_html: fn(String) -> Response,
    pub serve_search_results: fn(String) -> Response,
    pub serve_latex_svg: fn(String, String, String) -> Response,
}

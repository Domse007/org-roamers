pub mod types;

use rouille::Response;
use types::OrgAsHTMLResponse;

use crate::{server::Query, ServerState};

/// `APICalls` specify all operations the server supports when interacting with
/// the client. This is then used in [`crate::server::start_server`].
pub struct APICalls {
    /// What the server should return if the default route is called.
    pub default_route: fn(&mut ServerState, String, Option<String>) -> Response,
    /// What the server should return if the graph is requested.
    pub get_graph_data: fn(&mut ServerState) -> Response,
    /// What the server should return if some node is requested as html.
    pub get_org_as_html: fn(&mut ServerState, Query, String) -> OrgAsHTMLResponse,
    /// What the server should return for some query.
    pub serve_search_results: fn(&mut ServerState, String) -> Response,
    /// What the server should return if some inline latex should be rendered.
    pub serve_latex_svg: fn(&mut ServerState, String, String, String) -> Response,
}

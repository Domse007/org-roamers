pub mod types;

use std::sync::{Arc, Mutex};

use axum::response::Response;
use types::{GraphData, OrgAsHTMLResponse, SearchResponse, ServerStatus};

use crate::{
    perf::{PerfCollector, PerfPoint},
    server::Query,
    ServerState,
};

/// `APICalls` specify all operations the server supports when interacting with
/// the client. This is then used in [`crate::server::start_server`].
pub struct APICalls {
    /// What the server should return if the default route is called.
    pub default_route: fn(&mut ServerState, String, Option<String>) -> Response,
    /// What the server should return if the graph is requested.
    pub get_graph_data: fn(&mut ServerState) -> GraphData,
    /// What the server should return if some node is requested as html.
    pub get_org_as_html: fn(&mut ServerState, Query, String) -> OrgAsHTMLResponse,
    /// What the server should return for some query.
    pub serve_search_results: fn(&mut ServerState, String) -> SearchResponse,
    pub get_status_data: fn(&mut ServerState, Arc<Mutex<bool>>) -> ServerStatus,
    /// What the server should return if some inline latex should be rendered.
    pub serve_latex_svg: fn(&mut ServerState, String, String, String) -> Response,
}

pub(crate) struct APICallsInternal {
    default_route_perf: PerfCollector,
    default_route: fn(&mut ServerState, String, Option<String>) -> Response,
    get_graph_data_perf: PerfCollector,
    get_graph_data: fn(&mut ServerState) -> GraphData,
    get_org_as_html_perf: PerfCollector,
    get_org_as_html: fn(&mut ServerState, Query, String) -> OrgAsHTMLResponse,
    serve_search_results_perf: PerfCollector,
    serve_search_results: fn(&mut ServerState, String) -> SearchResponse,
    serve_latex_svg_perf: PerfCollector,
    serve_latex_svg: fn(&mut ServerState, String, String, String) -> Response,
    get_status_data_perf: PerfCollector,
    get_status_data: fn(&mut ServerState, Arc<Mutex<bool>>) -> ServerStatus,
}

impl APICallsInternal {
    pub fn default_route(
        &mut self,
        state: &mut ServerState,
        root: String,
        url: Option<String>,
    ) -> Response {
        let point = PerfPoint::new();
        let res = (self.default_route)(state, root, url);
        self.default_route_perf.submit(point);
        self.default_route_perf.report("default_route");
        res
    }
    pub fn get_graph_data(&mut self, state: &mut ServerState) -> GraphData {
        let point = PerfPoint::new();
        let res = (self.get_graph_data)(state);
        self.get_graph_data_perf.submit(point);
        self.get_graph_data_perf.report("get_graph_data");
        res
    }
    pub fn get_org_as_html(
        &mut self,
        state: &mut ServerState,
        query: Query,
        scope: String,
    ) -> OrgAsHTMLResponse {
        let point = PerfPoint::new();
        let res = (self.get_org_as_html)(state, query, scope);
        self.get_org_as_html_perf.submit(point);
        self.get_org_as_html_perf.report("get_org_as_html");
        res
    }
    pub fn serve_search_results(
        &mut self,
        state: &mut ServerState,
        query: String,
    ) -> SearchResponse {
        let point = PerfPoint::new();
        let res = (self.serve_search_results)(state, query);
        self.serve_search_results_perf.submit(point);
        self.serve_search_results_perf
            .report("serve_search_results");
        res
    }
    pub fn serve_latex_svg(
        &mut self,
        state: &mut ServerState,
        tex: String,
        color: String,
        id: String,
    ) -> Response {
        let point = PerfPoint::new();
        let res = (self.serve_latex_svg)(state, tex, color, id);
        self.serve_latex_svg_perf.submit(point);
        self.serve_latex_svg_perf.report("serve_latex_svg");
        res
    }
    pub fn get_status_data(
        &mut self,
        state: &mut ServerState,
        changes: Arc<Mutex<bool>>,
    ) -> ServerStatus {
        let point = PerfPoint::new();
        let res = (self.get_status_data)(state, changes);
        self.get_status_data_perf.submit(point);
        self.get_status_data_perf.report("get_status_data");
        res
    }
}

impl From<APICalls> for APICallsInternal {
    fn from(value: APICalls) -> Self {
        Self {
            default_route_perf: PerfCollector::new(),
            default_route: value.default_route,
            get_graph_data_perf: PerfCollector::new(),
            get_graph_data: value.get_graph_data,
            get_org_as_html_perf: PerfCollector::new(),
            get_org_as_html: value.get_org_as_html,
            serve_search_results_perf: PerfCollector::new(),
            serve_search_results: value.serve_search_results,
            serve_latex_svg_perf: PerfCollector::new(),
            serve_latex_svg: value.serve_latex_svg,
            get_status_data_perf: PerfCollector::new(),
            get_status_data: value.get_status_data,
        }
    }
}

use crate::search::Search;
use crate::server::types::{SearchResponse, SearchResponseProvider};
use crate::ServerState;

pub fn search(db: &mut ServerState, query: String) -> SearchResponse {
    let search = Search::new(query.as_str());
    let res = search.search(db.sqlite.connection());

    let nodes = match res {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("An error occurred while providing search: {err}");
            return SearchResponse { providers: vec![] };
        }
    };

    SearchResponse {
        providers: vec![SearchResponseProvider {
            source: "sqlite".to_string(),
            results: nodes,
        }],
    }
}

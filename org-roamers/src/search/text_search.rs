use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tokio_util::sync::CancellationToken;

use crate::{
    search::SearchResultSender,
    server::{
        types::{RoamID, RoamTitle},
        AppState,
    },
};

// TODO: make this configurable.
const THRESHOLD: i64 = 90;

pub struct FullTextSeach {
    pub(crate) cancel_token: CancellationToken,
    pub(crate) sender: SearchResultSender,
}

impl FullTextSeach {
    pub fn new(sender: SearchResultSender) -> Self {
        Self {
            sender,
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.sender.id()
    }

    pub fn cancel(&mut self) {
        self.cancel_token.cancel();
        // Create a new token for the next search
        self.cancel_token = CancellationToken::new();
    }

    pub async fn feed(&mut self, state: AppState, f: &super::Feeder) -> anyhow::Result<()> {
        let matcher = SkimMatcherV2::default();
        let query = f.s.to_string();
        let cancel_token = self.cancel_token.clone();

        const NODE_STMNT: &str = r#"
        SELECT title, id FROM nodes
        WHERE id = ?;
        "#;

        const TAGS_STMNT: &str = r#"
        SELECT tag FROM tags
        WHERE node_id = ?"#;

        let sender = self.sender.clone();

        tokio::spawn(async move {
            // Collect cache entries and clone sqlite pool before any async operations
            let (cache_entries, sqlite) = {
                let state = state.lock().unwrap();
                let cache_entries: Vec<_> = state.cache.iter()
                    .map(|(k, v)| (k.clone(), v.content().to_string()))
                    .collect();
                (cache_entries, state.sqlite.clone())
            };

            for (key, content) in cache_entries {
                if cancel_token.is_cancelled() {
                    return;
                }

                if let Some((score, _index_types)) = matcher.fuzzy_indices(&content, &query)
                {
                    if score >= THRESHOLD {
                        let (title, id): (String, String) = match sqlx::query_as(NODE_STMNT)
                            .bind(key.id())
                            .fetch_one(&sqlite)
                            .await
                        {
                            Ok(pair) => pair,
                            Err(_) => {
                                tracing::error!("No entry found for {}", key.id());
                                continue;
                            }
                        };

                        let (title, id) = (RoamTitle::from(title), RoamID::from(id));

                        let tags: Vec<String> = match sqlx::query_as(TAGS_STMNT)
                            .bind(id.id())
                            .fetch_all(&sqlite)
                            .await
                        {
                            Ok(tags) => tags.into_iter().map(|e: (String,)| e.0).collect(),
                            Err(err) => {
                                tracing::error!("An error occured: {err}");
                                vec![]
                            }
                        };

                        // TODO: preview not implemented.
                        if let Err(err) = sender.send(title, id, tags, None) {
                            tracing::error!("{err}");
                        };

                        if cancel_token.is_cancelled() {
                            return;
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

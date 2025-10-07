use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rusqlite::params;
use tokio_util::sync::CancellationToken;

use crate::{
    search::{Configuration, SearchResultSender},
    server::{types::{RoamID, RoamTitle}, AppState},
};

// TODO: make this configurable.
const THRESHOLD: i64 = 80;

pub struct FullTextSeach {
    cancel_token: CancellationToken,
}

impl FullTextSeach {
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
        }
    }
}

impl FullTextSeach {
    pub fn configuration(&self) -> super::Configuration {
        Configuration {
            returns_preview: true,
        }
    }

    pub fn cancel(&mut self) {
        self.cancel_token.cancel();
    }

    pub async fn feed(
        &mut self,
        state: AppState,
        sender: SearchResultSender,
        f: &super::Feeder,
    ) -> anyhow::Result<()> {
        let matcher = SkimMatcherV2::default();
        let query = f.s.to_string();
        let cancel_token = self.cancel_token.clone();

        const NODE_STMNT: &str = r#"
        SELECT title, id FROM nodes
        WHERE id = ?1;
        "#;

        const TAGS_STMNT: &str = r#"
        SELECT tag FROM tags
        WHERE node_id = ?1;"#;

        tokio::task::spawn_blocking(move || {
            let mut state = state.lock().unwrap();
            let ref mut state = *state;
            
            let mut sqlite = state.sqlite.lock().unwrap();
            let conn = sqlite.connection();
            let mut node_stmnt = conn.prepare(NODE_STMNT).unwrap();
            let mut tags_stmnt = conn.prepare(TAGS_STMNT).unwrap();

            for (key, value) in state.cache.iter() {
                if cancel_token.is_cancelled() {
                    return;
                }

                if let Some((score, _index_types)) = matcher.fuzzy_indices(value.content(), &query)
                {
                    if score >= THRESHOLD {
                        let (title, id): (RoamTitle, RoamID) =
                            match node_stmnt.query_map(params![key.id()], |row| {
                                Ok((
                                    RoamTitle::from(row.get_unwrap::<usize, String>(0)),
                                    RoamID::from(row.get_unwrap::<usize, String>(1)),
                                ))
                            }) {
                                Ok(pair) => match pair.map(|e| e.unwrap()).next() {
                                    Some(pair) => pair,
                                    None => {
                                        tracing::error!("No entry found for {}", key.id());
                                        continue;
                                    }
                                },
                                Err(err) => {
                                    tracing::error!("An error occured: {err}");
                                    continue;
                                }
                            };

                        let tags = match tags_stmnt.query_map(params![id.id()], |row| {
                            Ok(row.get_unwrap::<usize, String>(0))
                        }) {
                            Ok(tags) => tags.map(Result::unwrap).collect(),
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
        })
        .await?;

        Ok(())
    }
}

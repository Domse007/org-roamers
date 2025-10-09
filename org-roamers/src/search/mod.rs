use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    search::{default::DefaultSearch, text_search::FullTextSeach},
    server::{
        types::{RoamID, RoamTitle},
        AppState,
    },
};

mod default;
mod text_search;

pub struct Feeder {
    s: String,
}

impl Feeder {
    pub fn new(s: String) -> Self {
        Self { s }
    }
}

#[derive(Clone)]
pub struct SearchResultSender {
    provider_id: usize,
    sender: mpsc::Sender<SearchResultEntry>,
}

impl SearchResultSender {
    pub fn new(provider_id: usize, sender: mpsc::Sender<SearchResultEntry>) -> Self {
        Self {
            provider_id,
            sender,
        }
    }

    pub fn id(&self) -> usize {
        self.provider_id
    }

    pub fn send(
        &self,
        title: RoamTitle,
        id: RoamID,
        tags: Vec<String>,
        preview: Option<(String, usize, usize)>,
    ) -> anyhow::Result<()> {
        self.sender.try_send(SearchResultEntry {
            provider: self.provider_id,
            title,
            id,
            tags,
            preview,
        })?;
        Ok(())
    }
}

// TODO: move to src/server/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultEntry {
    provider: usize,
    pub title: RoamTitle,
    pub id: RoamID,
    pub tags: Vec<String>,
    /// `preview` is a tuple where:
    /// - the first element is the source line where the match occured.
    /// - the second and third element give the range where the matching exactly
    ///   happened.
    pub preview: Option<(String, usize, usize)>,
}

pub enum SearchProvider {
    FullTextSearch(FullTextSeach),
    DefaultSearch(DefaultSearch),
}

impl SearchProvider {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FullTextSearch(_) => "Full text search",
            Self::DefaultSearch(_) => "Default search",
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Self::FullTextSearch(fts) => fts.id(),
            Self::DefaultSearch(ds) => ds.id(),
        }
    }

    pub fn cancel(&mut self) {
        match self {
            Self::FullTextSearch(fts) => fts.cancel(),
            Self::DefaultSearch(_) => {
                // DefaultSearch doesn't have async operations to cancel
            }
        }
    }
}

pub struct SearchProviderList {
    providers: Vec<SearchProvider>,
}

impl SearchProviderList {
    pub fn new(sender: mpsc::Sender<SearchResultEntry>) -> Self {
        Self {
            providers: vec![
                SearchProvider::DefaultSearch(DefaultSearch::new(SearchResultSender::new(
                    0,
                    sender.clone(),
                ))),
                SearchProvider::FullTextSearch(FullTextSeach::new(SearchResultSender::new(
                    1, sender,
                ))),
            ],
        }
    }

    pub async fn feed(&mut self, state: AppState, f: Feeder) {
        let mut tasks = vec![];
        
        // We need to extract providers to spawn them in separate tasks
        // Since we can't easily do that with mutable references, we'll spawn tasks directly
        for provider in &mut self.providers {
            let state_clone = state.clone();
            let query = f.s.clone();
            
            // Spawn each provider's feed as a separate task
            let task = match provider {
                SearchProvider::DefaultSearch(ds) => {
                    let sender = ds.sender.clone();
                    tokio::spawn(async move {
                        // TODO: there appears to be no use for the Self::providers...
                        let mut ds = DefaultSearch::new(sender);
                        ds.feed(state_clone, &Feeder::new(query)).await
                    })
                }
                SearchProvider::FullTextSearch(fts) => {
                    let sender = fts.sender.clone();
                    let cancel_token = fts.cancel_token.clone();
                    tokio::spawn(async move {
                        let mut fts = FullTextSeach { sender, cancel_token };
                        fts.feed(state_clone, &Feeder::new(query)).await
                    })
                }
            };
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {},
                Ok(Err(err)) => tracing::error!("Search provider failed: {err}"),
                Err(err) => tracing::error!("Search provider task panicked: {err}"),
            }
        }
    }

    pub fn config(&self) -> Vec<(usize, String)> {
        let mut map = vec![];
        for provider in &self.providers {
            map.push((provider.id(), provider.name().to_string()));
        }
        map
    }

    /// Cancel all ongoing search operations.
    /// This should be called when starting a new search to avoid wasting resources.
    pub fn cancel(&mut self) {
        for provider in &mut self.providers {
            provider.cancel();
        }
    }
}


use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    search::{default::DefaultSearch, text_search::FullTextSeach},
    server::{types::{RoamID, RoamTitle}, AppState},
};

pub mod default;
mod text_search;

#[derive(Serialize)]
pub struct Configuration {
    pub returns_preview: bool,
}

pub struct Feeder {
    s: String,
}

impl Feeder {
    pub fn new(s: String) -> Self {
        Self { s }
    }
}

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

    pub fn send(
        &self,
        title: RoamTitle,
        id: RoamID,
        tags: Vec<String>,
        preview: Option<(String, usize, usize)>,
    ) -> anyhow::Result<()> {
        self.sender.blocking_send(SearchResultEntry {
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
    pub async fn feed(
        &mut self,
        state: AppState,
        sender: SearchResultSender,
        f: &Feeder,
    ) -> anyhow::Result<()> {
        match self {
            Self::FullTextSearch(fts) => fts.feed(state, sender, f).await,
            Self::DefaultSearch(ds) => ds.feed(state, sender, f).await,
        }
    }
}

pub struct SearchProviderList {
    providers: Vec<SearchProvider>,
}

impl SearchProviderList {
    pub fn new() -> Self {
        Self {
            providers: vec![
                SearchProvider::DefaultSearch(DefaultSearch),
                SearchProvider::FullTextSearch(FullTextSeach::new()),
            ],
        }
    }

    pub async fn feed(
        &mut self,
        state: AppState,
        sender: mpsc::Sender<SearchResultEntry>,
        f: Feeder,
    ) {
        let mut provider_id = 0;
        for provider in &mut self.providers {
            let search_sender = SearchResultSender::new(provider_id, sender.clone());
            if let Err(err) = provider.feed(state.clone(), search_sender, &f).await {
                tracing::error!("An error occured: {err}");
            }

            provider_id += 1;
        }
    }
}

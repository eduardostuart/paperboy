use chrono::{Duration, Utc};
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::Serialize;
use std::ops::Sub;

const HTTPCLIENT_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Serialize)]
pub struct Entry {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Feed {
    pub url: String,
    pub title: String,
    pub entries: Vec<Entry>,
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            url: Default::default(),
            entries: vec![],
        }
    }
}

impl Feed {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ..Self::default()
        }
    }

    pub async fn fetch(&self) -> crate::Result<Feed> {
        let content = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTPCLIENT_TIMEOUT_SECS))
            .build()?
            .get(&self.url)
            .send()
            .await?
            .bytes()
            .await?;

        match feed_rs::parser::parse(&content[..]) {
            Ok(result) => Ok(Self {
                url: String::from(&self.url),
                title: result.title.unwrap().content,
                entries: self.filter_items_from_yesterday(result.entries),
            }),
            Err(e) => {
                return Err(crate::error::Error::CouldNotParseRSSFromUrl(format!(
                    "{}: {}",
                    self.url,
                    e.to_string()
                )));
            }
        }
    }

    pub(self) fn filter_items_from_yesterday(
        &self,
        entries: Vec<feed_rs::model::Entry>,
    ) -> Vec<Entry> {
        let yesterday = Utc::now().sub(Duration::days(1));
        entries
            .into_iter()
            .filter(|e| match e.published {
                Some(published) => published.ge(&yesterday),
                None => false,
            })
            .map(|entry| Entry {
                url: entry.links.first().unwrap().href.clone(),
                title: entry.title.unwrap().content,
            })
            .collect::<Vec<Entry>>()
    }
}

#[derive(Debug)]
pub struct FeedLoader {
    pub concurrency: usize,
    pub subscriptions: Vec<String>,
}

#[derive(Debug)]
pub struct FeedLoadError {
    pub has_errors: bool,
    pub errors: Vec<String>,
}

impl FeedLoader {
    pub fn new(subscriptions: Vec<String>) -> Self {
        FeedLoader {
            concurrency: 12,
            subscriptions,
        }
    }

    pub async fn load(&self) -> Option<(Vec<Feed>, FeedLoadError)> {
        // Create an unordered buffered list of pending futures
        let mut st = stream::iter(&self.subscriptions)
            .map(|url| async move { Feed::new(url.clone()).fetch().await })
            .buffer_unordered(self.concurrency);

        // Check for each stream item and only return items that have entries
        let mut items = Vec::<Feed>::new();
        let mut errors = Vec::<String>::new();
        //
        while let Some(response) = st.next().await {
            match response {
                Ok(feed) => {
                    if !feed.entries.is_empty() {
                        items.push(feed)
                    }
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            };
        }

        if !items.is_empty() {
            Some((
                items,
                FeedLoadError {
                    has_errors: !errors.is_empty(),
                    errors,
                },
            ))
        } else {
            None
        }
    }
}

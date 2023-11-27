use chrono::{Duration, Utc};
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use std::ops::Sub;

const HTTPCLIENT_TIMEOUT_SECS: u64 = 3;
const HTTPCLIENT_CONNECTION_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Serialize, Clone)]
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

    pub async fn fetch(self) -> crate::Result<Feed> {
        log::debug!("Fetching url {}", &self.url);

        let content = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTPCLIENT_TIMEOUT_SECS))
            .connect_timeout(std::time::Duration::from_secs(
                HTTPCLIENT_CONNECTION_TIMEOUT_SECS,
            ))
            .build()?
            .get(&self.url)
            // See: https://stackoverflow.com/a/7001617/5155484
            .header(
                "Accept",
                "application/rss+xml, application/rdf+xml, application/atom+xml, application/feed+json, application/xml;q=0.9, text/xml;q=0.8"
            )
            .header(
                "User-Agent",
                "Paperboy (https://github.com/eduardostuart/paperboy)"
            )
            .send()
            .await?
            .bytes()
            .await?;

        match feed_rs::parser::parse(&content[..]) {
            Ok(result) => {
                log::debug!("Fetch Result {:?}", result);
                Ok(Feed {
                    url: String::from(&self.url),
                    title: result.title.unwrap().content,
                    entries: self.filter_items_from_yesterday(result.entries).to_vec(),
                })
            }
            Err(e) => {
                log::debug!("Fetch result error {}", e);
                Err(crate::error::Error::CouldNotParseRSSFromUrl(format!(
                    "{}: {}",
                    self.url, e
                )))
            }
        }
    }

    pub(self) fn filter_items_from_yesterday(
        &self,
        entries: Vec<feed_rs::model::Entry>,
    ) -> Vec<Entry> {
        let yesterday = Utc::now().sub(Duration::days(1));
        log::trace!("Filtering items from yesterday {}", yesterday);
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
    pub subscriptions: Vec<String>,
}

#[derive(Debug)]
pub struct FeedLoadError {
    pub has_errors: bool,
    pub errors: Vec<String>,
}

impl FeedLoader {
    pub fn new(subscriptions: Vec<String>) -> Self {
        FeedLoader { subscriptions }
    }

    pub async fn load(&self) -> Option<(Vec<Feed>, FeedLoadError)> {
        log::trace!("Loading {} subscriptions", self.subscriptions.len());

        let mut futures = futures::stream::iter(self.subscriptions.to_owned())
            .map(|url| tokio::spawn(async move { Feed::new(url).fetch().await }))
            .buffer_unordered(10);

        let mut items = Vec::new();
        let mut errors = Vec::new();

        while let Some(f) = futures.next().await {
            match f {
                Ok(Ok(feed)) => {
                    log::debug!("Feed {} has {} items", feed.url, feed.entries.len());

                    if !feed.entries.is_empty() {
                        items.push(feed);
                    }
                }
                Ok(Err(e)) => {
                    log::error!("Error while loading feed {}", e);
                    errors.push(e.to_string());
                }
                Err(e) => {
                    log::error!("Error {}", e);
                    errors.push(e.to_string());
                }
            }
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
            log::debug!("There are no new entries");
            None
        }
    }
}

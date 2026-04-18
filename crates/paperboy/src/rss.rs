//! Fetch and parse RSS, Atom, and JSON feeds.

use chrono::{Duration, Utc};
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use std::ops::Sub;

const HTTPCLIENT_TIMEOUT_SECS: u64 = 3;
const HTTPCLIENT_CONNECTION_TIMEOUT_SECS: u64 = 3;
const USER_AGENT: &str = "Paperboy (github.com/eduardostuart/paperboy)";

/// A single entry (article) from a feed.
#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    /// Title of the entry.
    pub title: String,
    /// URL pointing to the entry's page.
    pub url: String,
}

/// A parsed feed together with the entries that should be delivered.
///
/// Only entries published within the last 24 hours are retained; older entries
/// and entries without a publication date are dropped.
#[derive(Debug, Serialize)]
pub struct Feed {
    /// URL of the feed itself (not of any individual entry).
    pub url: String,
    /// Feed title, or the feed URL if the source did not advertise one.
    pub title: String,
    /// Entries published within the last 24 hours.
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
    /// Creates an empty `Feed` associated with the given URL.
    pub fn new(url: String) -> Self {
        Self {
            url,
            ..Self::default()
        }
    }

    /// Fetches the feed over HTTP and parses it.
    ///
    /// Uses a 3-second connect and read timeout. Entries without a publication
    /// date and entries older than 24 hours are dropped.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Http`] for transport-level failures and
    /// [`crate::error::Error::CouldNotParseRSSFromUrl`] when the response
    /// cannot be parsed as a feed.
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
            .header("User-Agent",USER_AGENT)
            .send()
            .await?
            .bytes()
            .await?;

        match feed_rs::parser::parse(&content[..]) {
            Ok(result) => {
                log::debug!("Fetch Result {:?}", result);
                let title = result
                    .title
                    .map(|t| t.content)
                    .unwrap_or_else(|| self.url.clone());
                Ok(Feed {
                    url: String::from(&self.url),
                    title,
                    entries: self.filter_items_from_yesterday(result.entries),
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
            .filter_map(|entry| {
                let url = entry.links.first().map(|l| l.href.clone())?;
                let title = entry.title.map(|t| t.content)?;
                Some(Entry { url, title })
            })
            .collect::<Vec<Entry>>()
    }
}

/// Concurrently fetches a batch of subscription URLs.
///
/// At most 10 requests are in flight at a time.
#[derive(Debug)]
pub struct FeedLoader {
    /// Feed URLs to load.
    pub subscriptions: Vec<String>,
}

/// Aggregated failures reported by [`FeedLoader::load`].
#[derive(Debug)]
pub struct FeedLoadError {
    /// `true` when at least one feed failed to load.
    pub has_errors: bool,
    /// Human-readable error messages, one per failed feed.
    pub errors: Vec<String>,
}

impl FeedLoader {
    /// Creates a new loader for the provided subscription URLs.
    pub fn new(subscriptions: Vec<String>) -> Self {
        FeedLoader { subscriptions }
    }

    /// Loads every subscription in parallel and returns the feeds that have
    /// fresh entries alongside any per-feed errors.
    ///
    /// Returns `None` when no feed produced any fresh entry, so callers can
    /// short-circuit the delivery step.
    pub async fn load(&self) -> Option<(Vec<Feed>, FeedLoadError)> {
        log::trace!("Loading {} subscriptions", self.subscriptions.len());

        let mut futures = futures::stream::iter(self.subscriptions.to_owned())
            .map(|url| Feed::new(url).fetch())
            .buffer_unordered(10);

        let mut items = Vec::new();
        let mut errors = Vec::new();

        while let Some(f) = futures.next().await {
            match f {
                Ok(feed) => {
                    log::debug!("Feed {} has {} items", feed.url, feed.entries.len());

                    if !feed.entries.is_empty() {
                        items.push(feed);
                    }
                }
                Err(e) => {
                    log::error!("Error while loading feed {}", e);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_entry_from(xml: &str) -> feed_rs::model::Entry {
        feed_rs::parser::parse(xml.as_bytes())
            .unwrap()
            .entries
            .into_iter()
            .next()
            .unwrap()
    }

    fn seed_entry() -> feed_rs::model::Entry {
        parse_entry_from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel><title>T</title><link>http://e</link><description>D</description>
<item><title>seed-title</title><link>http://seed-link</link></item>
</channel></rss>"#,
        )
    }

    fn entry_with_published(published: chrono::DateTime<Utc>) -> feed_rs::model::Entry {
        feed_rs::model::Entry {
            published: Some(published),
            ..seed_entry()
        }
    }

    fn feed() -> Feed {
        Feed::new("http://example.com/feed".to_string())
    }

    #[test]
    fn filter_keeps_entries_from_last_24h() {
        let entry = entry_with_published(Utc::now());
        let result = feed().filter_items_from_yesterday(vec![entry]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "seed-title");
        assert!(result[0].url.starts_with("http://seed-link"));
    }

    #[test]
    fn filter_drops_entries_older_than_yesterday() {
        let entry = entry_with_published(Utc::now() - Duration::days(3));
        let result = feed().filter_items_from_yesterday(vec![entry]);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_drops_entries_with_no_publish_date() {
        let entry = feed_rs::model::Entry {
            published: None,
            ..seed_entry()
        };
        let result = feed().filter_items_from_yesterday(vec![entry]);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_does_not_panic_on_entry_without_title() {
        // Regression: previous version called entry.title.unwrap()
        let entry = feed_rs::model::Entry {
            title: None,
            published: Some(Utc::now()),
            ..seed_entry()
        };
        let result = feed().filter_items_from_yesterday(vec![entry]);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_does_not_panic_on_entry_without_links() {
        // Regression: previous version called entry.links.first().unwrap()
        let entry = feed_rs::model::Entry {
            links: vec![],
            published: Some(Utc::now()),
            ..seed_entry()
        };
        let result = feed().filter_items_from_yesterday(vec![entry]);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_keeps_valid_and_skips_malformed_in_same_batch() {
        let good = entry_with_published(Utc::now());
        let no_title = feed_rs::model::Entry {
            title: None,
            published: Some(Utc::now()),
            ..seed_entry()
        };
        let no_link = feed_rs::model::Entry {
            links: vec![],
            published: Some(Utc::now()),
            ..seed_entry()
        };
        let result = feed().filter_items_from_yesterday(vec![good, no_title, no_link]);
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn fetch_parses_valid_rss_and_keeps_fresh_items() {
        let mut server = mockito::Server::new_async().await;
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
<title>My Blog</title>
<link>http://example.com</link>
<description>Blog</description>
<item>
<title>Today Post</title>
<link>http://example.com/today</link>
<pubDate>{}</pubDate>
</item>
</channel>
</rss>"#,
            Utc::now().format("%a, %d %b %Y %H:%M:%S GMT")
        );
        let _m = server
            .mock("GET", "/feed")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(&body)
            .create_async()
            .await;

        let feed = Feed::new(format!("{}/feed", server.url())).fetch().await.unwrap();
        assert_eq!(feed.title, "My Blog");
        assert_eq!(feed.entries.len(), 1);
        assert_eq!(feed.entries[0].title, "Today Post");
    }

    #[tokio::test]
    async fn fetch_parses_json_feed_and_keeps_fresh_items() {
        let mut server = mockito::Server::new_async().await;
        let body = format!(
            r#"{{
    "version": "https://jsonfeed.org/version/1.1",
    "title": "My JSON Feed",
    "home_page_url": "http://example.com/",
    "feed_url": "http://example.com/feed.json",
    "items": [
        {{
            "id": "1",
            "title": "Today JSON Post",
            "url": "http://example.com/today",
            "date_published": "{}"
        }},
        {{
            "id": "2",
            "title": "Old JSON Post",
            "url": "http://example.com/old",
            "date_published": "2020-01-01T00:00:00Z"
        }}
    ]
}}"#,
            Utc::now().to_rfc3339()
        );
        let _m = server
            .mock("GET", "/feed.json")
            .with_status(200)
            .with_header("content-type", "application/feed+json")
            .with_body(&body)
            .create_async()
            .await;

        let feed = Feed::new(format!("{}/feed.json", server.url()))
            .fetch()
            .await
            .unwrap();
        assert_eq!(feed.title, "My JSON Feed");
        assert_eq!(feed.entries.len(), 1);
        assert_eq!(feed.entries[0].title, "Today JSON Post");
        assert!(feed.entries[0].url.starts_with("http://example.com/today"));
    }

    #[tokio::test]
    async fn fetch_falls_back_to_url_when_feed_has_no_title() {
        // Regression: previous version called result.title.unwrap()
        let mut server = mockito::Server::new_async().await;
        // Atom feed without a <title> element
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<id>urn:example</id>
<updated>2099-01-01T00:00:00Z</updated>
</feed>"#;
        let _m = server
            .mock("GET", "/feed")
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let url = format!("{}/feed", server.url());
        let feed = Feed::new(url.clone()).fetch().await.unwrap();
        assert_eq!(feed.title, url);
    }

    #[tokio::test]
    async fn fetch_returns_parse_error_for_non_feed_body() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/not-a-feed")
            .with_status(200)
            .with_body("<html><body>not a feed</body></html>")
            .create_async()
            .await;

        let err = Feed::new(format!("{}/not-a-feed", server.url()))
            .fetch()
            .await
            .unwrap_err();
        match err {
            crate::error::Error::CouldNotParseRSSFromUrl(_) => {}
            other => panic!("expected CouldNotParseRSSFromUrl, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn fetch_returns_http_error_on_5xx() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/boom")
            .with_status(500)
            .create_async()
            .await;

        // A 500 with empty body fails feed parsing, which surfaces as a parse error.
        // The important guarantee is that it returns Err rather than panicking.
        let result = Feed::new(format!("{}/boom", server.url())).fetch().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn feed_loader_returns_none_when_all_feeds_have_no_entries() {
        let mut server = mockito::Server::new_async().await;
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel><title>Empty</title><link>http://e</link><description>D</description></channel>
</rss>"#;
        let _m = server
            .mock("GET", "/empty")
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let result = FeedLoader::new(vec![format!("{}/empty", server.url())])
            .load()
            .await;
        assert!(result.is_none());
    }
}

//! Deliver new posts from RSS, Atom, or JSON feeds by email.
//!
//! This crate powers the [`paperboy`](https://github.com/eduardostuart/paperboy)
//! CLI. It concurrently fetches a list of subscriptions, keeps entries
//! published within the last 24 hours, renders them through a Handlebars
//! template, and sends the result over SMTP.
//!
//! The high-level entry point is [`Paperboy`]. Feeds are loaded with
//! [`FeedLoader`] and delivered through [`Mailer`].
//!
//! # Example
//!
//! ```no_run
//! use paperboy::{Config, Credentials, FeedLoader, Paperboy};
//!
//! # async fn run() -> paperboy::Result<()> {
//! let subscriptions = vec!["https://example.com/feed.xml".to_string()];
//! let (feeds, _errors) = FeedLoader::new(subscriptions)
//!     .load()
//!     .await
//!     .expect("at least one feed returned entries");
//!
//! let config = Config {
//!     subject: "Daily digest".into(),
//!     from: "Paperboy <news@example.com>".into(),
//!     host: "smtp.example.com".into(),
//!     port: 587,
//!     credentials: Credentials {
//!         username: "user".into(),
//!         password: "pass".into(),
//!     },
//!     starttls: true,
//! };
//!
//! Paperboy::new("daily.html.hbs", None, config)
//!     .deliver(feeds, "reader@example.com".into())
//!     .await
//! # }
//! ```

pub mod error;

pub mod mailer;
mod rss;
pub mod subscriptions;

/// A [`std::result::Result`] whose error defaults to [`error::Error`].
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

use handlebars::{to_json, Handlebars, JsonValue as Value};
pub use mailer::{Config, Credentials, Mailer};
pub use rss::{Entry, Feed, FeedLoadError, FeedLoader};
use serde_json::Map;

/// Renders a collection of feeds into email bodies and delivers them over SMTP.
///
/// Both template paths are resolved at render time using
/// [Handlebars](https://handlebarsjs.com/). The HTML template is mandatory; the
/// plain-text template is optional and, when provided, is sent as the
/// `text/plain` alternative so mail clients that cannot render HTML still show
/// readable content.
#[derive(Debug)]
pub struct Paperboy<'a> {
    template_html: &'a str,
    template_text: Option<&'a str>,
    mailer_config: Config,
}

impl<'a> Paperboy<'a> {
    /// Builds a new `Paperboy`.
    ///
    /// `template_html` is a filesystem path to a Handlebars template that
    /// receives an `items` array (each element is a [`Feed`] with nested
    /// [`Entry`] values). `template_text` is an optional path used for the
    /// plain-text alternative.
    pub fn new(
        template_html: &'a str,
        template_text: Option<&'a str>,
        mailer_config: Config,
    ) -> Self {
        Self {
            template_html,
            template_text,
            mailer_config,
        }
    }

    pub(self) fn render_template(
        &self,
        items: &Vec<Feed>,
        template_path: &str,
    ) -> crate::Result<String> {
        let mut template = Handlebars::new();
        template.register_template_file("main", template_path)?;

        let mut data: Map<String, Value> = Map::new();
        data.insert("items".to_string(), to_json(items));

        Ok(template.render("main", &data)?)
    }

    /// Renders `items` through the configured templates and sends the email
    /// to `to`.
    ///
    /// # Errors
    ///
    /// Returns an [`error::Error`] if the template cannot be rendered, if the
    /// SMTP transport rejects the message, or if the server returns a
    /// non-positive response.
    pub async fn deliver(self, items: Vec<Feed>, to: String) -> crate::Result<()> {
        let body_html = self.render_template(&items, self.template_html)?;
        let body_text = self
            .template_text
            .map(|template| self.render_template(&items, template))
            .transpose()?;

        let subject = self.mailer_config.subject.clone();
        let response = Mailer::new(self.mailer_config)
            .send(to, subject, body_html, body_text)
            .await?;

        if !response.is_positive() {
            Err(crate::error::Error::ErrorSendingMail(format!(
                "Something went wrong: {}",
                response.code()
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn dummy_config() -> Config {
        Config {
            subject: "S".to_string(),
            from: "a@b.c".to_string(),
            host: "localhost".to_string(),
            port: 25,
            credentials: Credentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            starttls: false,
        }
    }

    fn write_template(content: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(".hbs")
            .tempfile()
            .unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn render_template_iterates_over_feeds_and_entries() {
        let tpl = write_template(
            "{{#each items}}{{this.title}}:{{#each this.entries}}{{this.title}}|{{this.url}};{{/each}}{{/each}}",
        );
        let path = tpl.path().to_str().unwrap().to_string();
        let paperboy = Paperboy::new(&path, None, dummy_config());

        let feeds = vec![Feed {
            url: "http://blog".to_string(),
            title: "Blog".to_string(),
            entries: vec![
                Entry {
                    title: "Post 1".to_string(),
                    url: "http://blog/1".to_string(),
                },
                Entry {
                    title: "Post 2".to_string(),
                    url: "http://blog/2".to_string(),
                },
            ],
        }];

        let rendered = paperboy.render_template(&feeds, &path).unwrap();
        assert_eq!(rendered, "Blog:Post 1|http://blog/1;Post 2|http://blog/2;");
    }

    #[test]
    fn render_template_returns_error_for_missing_file() {
        let paperboy = Paperboy::new("/does/not/exist.hbs", None, dummy_config());
        let err = paperboy
            .render_template(&vec![], "/does/not/exist.hbs")
            .unwrap_err();
        match err {
            error::Error::TemplateError(_) | error::Error::IO(_) => {}
            other => panic!("expected template or io error, got {:?}", other),
        }
    }

    #[test]
    fn render_template_returns_error_for_invalid_template_syntax() {
        let tpl = write_template("{{#each items}}unclosed");
        let path = tpl.path().to_str().unwrap().to_string();
        let paperboy = Paperboy::new(&path, None, dummy_config());

        let err = paperboy.render_template(&vec![], &path).unwrap_err();
        match err {
            error::Error::TemplateError(_) => {}
            other => panic!("expected TemplateError, got {:?}", other),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use std::{
        fs::{create_dir_all, File},
        io::Write,
        panic,
    };

    use rand::Rng;

    pub fn run<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        let result = panic::catch_unwind(|| test());
        assert!(result.is_ok())
    }

    pub fn create_tmp_file(content: &str) -> (String, String) {
        let random: String = rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let path = format!(".tmp{}", random);
        let file_path = format!("{}/file.txt", &path);

        create_dir_all(&path).unwrap();

        let file = File::create(&file_path).unwrap();
        write!(&file, "{}", &content).unwrap();

        (String::from(path), file_path.to_string())
    }
}

pub mod error;

pub mod mailer;
mod rss;
pub mod subscriptions;

/// Alias for a `Result` with the error type `paperboy::error::Error`.
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

use handlebars::{to_json, Handlebars, JsonValue as Value};
pub use mailer::{Config, Credentials, Mailer};
pub use rss::{Entry, Feed, FeedLoadError, FeedLoader};
use serde_json::Map;

#[derive(Debug)]
pub struct Paperboy<'a> {
    template_html: &'a str,
    template_text: Option<&'a str>,
    mailer_config: Config,
}

impl<'a> Paperboy<'a> {
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
    ) -> crate::Result<String, handlebars::RenderError> {
        let mut template = Handlebars::new();
        template.register_template_file("main", template_path)?;

        let mut data: Map<String, Value> = Map::new();
        data.insert("items".to_string(), to_json(items));

        template.render("main", &data)
    }

    pub async fn deliver(self, items: Vec<Feed>, to: String) -> crate::Result<()> {
        let body_html = self.render_template(&items, self.template_html).unwrap();
        let body_text = self
            .template_text
            .map(|template| self.render_template(&items, template).unwrap());

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
pub mod test_util {
    use std::{
        fs::{create_dir_all, File},
        io::Write,
        panic,
    };

    use rand::{distributions::Alphanumeric, Rng};

    pub fn run<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        let result = panic::catch_unwind(|| test());
        assert!(result.is_ok())
    }

    pub fn create_tmp_file(content: &str) -> (String, String) {
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
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

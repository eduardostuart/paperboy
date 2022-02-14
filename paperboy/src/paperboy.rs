use handlebars::{to_json, Handlebars};
use lettre::message::Mailbox;
use serde_json::{Map, Value};

use crate::{
    mailer::{Config as MailerConfig, Mailer},
    rss::Feed,
};

#[derive(Debug)]
pub struct Paperboy<'a> {
    template: &'a str,
    mailer_config: MailerConfig,
}

impl<'a> Paperboy<'a> {
    pub fn new(template: &'a str, mailer_config: MailerConfig) -> Self {
        Self {
            template,
            mailer_config,
        }
    }

    pub(self) fn render_template(
        &self,
        items: Vec<Feed>,
    ) -> crate::Result<String, handlebars::RenderError> {
        let mut template = Handlebars::new();
        template.register_template_file("main", &self.template)?;

        let mut data: Map<String, Value> = Map::new();
        data.insert("items".to_string(), to_json(items));

        template.render("main", &data)
    }

    pub async fn deliver(self, items: Vec<Feed>, to: String) -> crate::Result<()> {
        let body = self.render_template(items).unwrap();

        let to_mailbox = to.parse::<Mailbox>()?;

        let response = Mailer::new(self.mailer_config)
            .send(to_mailbox, "RSS Daily".to_string(), body)
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

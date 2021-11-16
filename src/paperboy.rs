use handlebars::{to_json, Handlebars};
use lettre::{message::Mailbox, transport::smtp::response::Response};
use serde_json::{Map, Value};

use crate::{
    mailer::{Mailer, MailerConfig},
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

    pub async fn deliver(self, items: Vec<Feed>, to: Mailbox) -> crate::Result<Response> {
        let body = self.render_template(items).unwrap();

        let result = Mailer::new(self.mailer_config)
            .send(to, "RSS Daily".to_string(), body)
            .await?;

        Ok(result)
    }
}

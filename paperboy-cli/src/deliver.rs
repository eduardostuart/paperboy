use paperboy::{
    subscriptions::load_from_file as load_subscriptions_from_file, Config, Credentials, FeedLoader,
    Paperboy, Result,
};

#[derive(Debug)]
pub struct MailConfig<'a> {
    pub mail_subject: &'a str,
    pub smtp_from: &'a str,
    pub smtp_username: &'a str,
    pub smtp_password: &'a str,
    pub smtp_host: &'a str,
    pub smtp_port: &'a u16,
}

#[derive(Debug)]
pub struct Deliver<'a> {
    pub subscription_file: &'a str,
    pub template_html: &'a str,
    pub template_text: Option<&'a str>,
    pub mail_config: MailConfig<'a>,
}

#[derive(Debug)]
pub struct DeliverResult {
    pub delivered: bool,
    pub message: String,
    pub errors: Option<Vec<String>>,
}

impl<'a> Deliver<'a> {
    pub fn new(
        subscription_file: &'a str,
        template_html: &'a str,
        template_text: Option<&'a str>,
        mail_config: MailConfig<'a>,
    ) -> Self {
        Self {
            template_html,
            template_text,
            subscription_file,
            mail_config,
        }
    }

    pub async fn handle(&'a self, to: &'a str) -> Result<DeliverResult> {
        let subscriptions = load_subscriptions_from_file(self.subscription_file);

        log::info!("{} subscriptions loaded", subscriptions.len());

        let mailer_config = Config {
            subject: self.mail_config.mail_subject.to_string(),
            from: self.mail_config.smtp_from.to_string(),
            credentials: Credentials {
                username: self.mail_config.smtp_username.to_string(),
                password: self.mail_config.smtp_password.to_string(),
            },
            host: self.mail_config.smtp_host.to_string(),
            port: *self.mail_config.smtp_port,
        };

        log::debug!("Fetching latest posts from all subscriptions...");

        match FeedLoader::new(subscriptions).load().await {
            Some((items, error_result)) => {
                Paperboy::new(self.template_html, self.template_text, mailer_config)
                    .deliver(items, to.to_string())
                    .await?;

                let errors = if error_result.has_errors {
                    Some(error_result.errors)
                } else {
                    None
                };

                Ok(DeliverResult {
                    delivered: true,
                    message: "OK".to_string(),
                    errors,
                })
            }
            None => Ok(DeliverResult {
                delivered: false,
                message: "Nothing new for today".to_string(),
                errors: None,
            }),
        }
    }
}

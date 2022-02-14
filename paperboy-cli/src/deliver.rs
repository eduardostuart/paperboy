use paperboy::{
    subscriptions::load_from_file as load_subscriptions_from_file, Config, Credentials, FeedLoader,
    Paperboy, Result,
};

#[derive(Debug)]
pub struct MailConfig<'a> {
    pub smtp_from: &'a str,
    pub smtp_username: &'a str,
    pub smtp_password: &'a str,
    pub smtp_relay: &'a str,
}

#[derive(Debug)]
pub struct Deliver<'a> {
    pub subscription_file: &'a str,
    pub template: &'a str,
    pub to: &'a str,
    pub mail_config: MailConfig<'a>,
}

#[derive(Debug)]
pub struct DeliverResult {
    pub delivered: bool,
    pub message: String,
}

impl<'a> Deliver<'a> {
    pub fn new(subscription_file: &'a str, to: &'a str, mail_config: MailConfig<'a>) -> Self {
        Self {
            template: "emails/daily_email.hbs",
            subscription_file,
            to,
            mail_config,
        }
    }

    pub async fn handle(&'a self) -> Result<DeliverResult> {
        let subscriptions = load_subscriptions_from_file(&self.subscription_file);

        let mailer_config = Config {
            from: self.mail_config.smtp_from.to_string(),
            credentials: Credentials {
                username: self.mail_config.smtp_username.to_string(),
                password: self.mail_config.smtp_password.to_string(),
            },
            relay: self.mail_config.smtp_relay.to_string(),
        };

        let result = FeedLoader::new(subscriptions).load().await;

        if result.is_none() {
            Ok(DeliverResult {
                delivered: false,
                message: "Nothing new for today".to_string(),
            })
        } else {
            Paperboy::new(&self.template, mailer_config)
                .deliver(result.unwrap(), self.to.to_string())
                .await?;

            Ok(DeliverResult {
                delivered: true,
                message: "OK".to_string(),
            })
        }
    }
}

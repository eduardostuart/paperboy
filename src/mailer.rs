use lettre::message::{header, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};

#[derive(Debug)]
pub struct MailerConfig {
    pub from: String,
    pub relay: String,
    pub credentials: Credentials,
}

#[derive(Debug)]
pub struct Mailer {
    pub config: MailerConfig,
}

impl Mailer {
    pub fn new(config: MailerConfig) -> Self {
        Self { config }
    }

    pub async fn send(
        &self,
        to: Mailbox,
        subject: String,
        content: String,
    ) -> crate::Result<Response> {
        let singlepart = SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(content);

        let email = Message::builder()
            .from(self.config.from.parse::<Mailbox>().unwrap())
            .to(to)
            .subject(subject)
            .singlepart(singlepart)
            .unwrap();

        let response = SmtpTransport::relay(&self.config.relay)
            .unwrap()
            .credentials(self.config.credentials.clone())
            .build()
            .send(&email)?;

        Ok(response)
    }
}

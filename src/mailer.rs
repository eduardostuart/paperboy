use lettre::message::{header, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};

#[derive(Debug)]
pub struct MailerConfig {
    pub from: String,
    pub credentials: Credentials,
    pub relay: String,
}

#[derive(Debug)]
pub struct Mailer {
    pub config: MailerConfig,
}

impl Mailer {
    pub fn new(config: MailerConfig) -> Self {
        Self { config }
    }

    pub fn get_relay(&self) -> &str {
        &self.config.relay
    }

    pub fn get_credentials(self) -> Credentials {
        self.config.credentials
    }

    pub fn get_from(&self) -> &str {
        &self.config.from
    }

    pub async fn send(
        self,
        to: Mailbox,
        subject: String,
        content: String,
    ) -> crate::Result<Response> {
        let email = Message::builder()
            .from(self.get_from().parse::<Mailbox>().unwrap())
            .to(to)
            .subject(subject)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(content),
            )
            .unwrap();

        let response = SmtpTransport::relay(self.get_relay())
            .unwrap()
            .credentials(self.get_credentials())
            .build()
            .send(&email)?;

        Ok(response)
    }
}

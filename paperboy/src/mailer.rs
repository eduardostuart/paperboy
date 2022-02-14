use lettre::message::{header, SinglePart};
use lettre::transport::smtp::authentication::Credentials as LettreSmtpCredentials;
use lettre::transport::smtp::response::Response;
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub from: String,
    pub relay: String,
    pub credentials: Credentials,
}

#[derive(Debug)]
pub struct Mailer {
    pub config: Config,
}

impl Mailer {
    pub fn new(config: Config) -> Self {
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

        let credentials = LettreSmtpCredentials::new(
            self.config.credentials.username.clone(),
            self.config.credentials.password.clone(),
        );

        let response = SmtpTransport::relay(&self.config.relay)
            .unwrap()
            .credentials(credentials)
            .build()
            .send(&email)?;

        Ok(response)
    }
}

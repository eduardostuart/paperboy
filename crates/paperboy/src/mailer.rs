use lettre::message::header;
use lettre::message::MultiPart;
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials as LettreCredentials;
use lettre::transport::smtp::response::Response;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub subject: String,
    pub from: String,
    pub host: String,
    pub port: u16,
    pub credentials: Credentials,
    pub starttls: bool,
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
        to: String,
        subject: String,
        content_as_html: String,
        content_as_text: Option<String>,
    ) -> crate::Result<Response> {
        let parts = match content_as_text {
            Some(content_as_text) => MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(content_as_text),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(content_as_html),
                ),
            None => MultiPart::alternative().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(content_as_html),
            ),
        };

        let email = Message::builder()
            .from(self.config.from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .multipart(parts)
            .unwrap();

        let creds = LettreCredentials::new(
            self.config.credentials.username.clone(),
            self.config.credentials.password.clone(),
        );

        let transport = if self.config.starttls {
            SmtpTransport::starttls_relay(&self.config.host)
                .unwrap()
                .port(self.config.port)
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&self.config.host)
                .port(self.config.port)
                .credentials(creds)
                .build()
        };

        Ok(transport.send(&email)?)
    }
}

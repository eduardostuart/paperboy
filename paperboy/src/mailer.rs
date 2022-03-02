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
    pub from: String,
    pub host: String,
    pub port: u16,
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
        to: String,
        subject: String,
        content: String,
    ) -> crate::Result<Response> {
        let email = Message::builder()
            .from(self.config.from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(content),
                ),
            )
            .unwrap();

        let transport = SmtpTransport::starttls_relay(&*self.config.host)
            .unwrap()
            .credentials(LettreCredentials::new(
                self.config.credentials.username.clone(),
                self.config.credentials.password.clone(),
            ))
            .build();

        Ok(transport.send(&email)?)
    }
}

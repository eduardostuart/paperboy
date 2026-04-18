//! SMTP delivery built on top of [`lettre`].

use lettre::message::header;
use lettre::message::MultiPart;
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials as LettreCredentials;
use lettre::transport::smtp::response::Response;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;

/// SMTP authentication credentials.
#[derive(Debug)]
pub struct Credentials {
    /// SMTP username.
    pub username: String,
    /// SMTP password.
    pub password: String,
}

/// Configuration for the [`Mailer`].
#[derive(Debug)]
pub struct Config {
    /// Default subject line.
    pub subject: String,
    /// `From` header (e.g. `"Paperboy <news@example.com>"`).
    pub from: String,
    /// SMTP host (e.g. `"smtp.example.com"`).
    pub host: String,
    /// SMTP port (commonly 25, 465, or 587).
    pub port: u16,
    /// SMTP authentication credentials.
    pub credentials: Credentials,
    /// When `true`, upgrade the connection with STARTTLS. Set to `false` for
    /// local development servers that do not offer TLS.
    pub starttls: bool,
}

/// Sends multipart (`text/plain` + `text/html`) emails through an SMTP relay.
#[derive(Debug)]
pub struct Mailer {
    /// SMTP configuration used by this mailer.
    pub config: Config,
}

impl Mailer {
    /// Creates a new `Mailer` with the given [`Config`].
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Sends an email to `to` with the provided HTML body and an optional
    /// plain-text alternative.
    ///
    /// When `content_as_text` is `Some`, the message is sent as
    /// `multipart/alternative` with the plain-text part listed first (per RFC
    /// 2046 simplest-first ordering). When it is `None`, only the HTML part is
    /// sent.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::error::Error::SmtpError`] when the SMTP transport
    /// fails and [`crate::error::Error::MailContentError`] when the message
    /// cannot be assembled.
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

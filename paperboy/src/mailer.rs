use lettre::smtp::authentication::Credentials as LettreCredentials;
use lettre::smtp::response::Response;
use lettre::SmtpClient;
use lettre::Transport;
use lettre_email::EmailBuilder;
use lettre_email::Mailbox;

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
        let mut transport = SmtpClient::new_simple(&*self.config.host)
            .unwrap()
            .credentials(LettreCredentials::new(
                self.config.credentials.username.clone(),
                self.config.credentials.password.clone(),
            ))
            .transport();

        Ok(transport.send(
            EmailBuilder::new()
                .from(self.config.from.parse::<Mailbox>().unwrap())
                .to(to.parse::<Mailbox>().unwrap())
                .subject(subject)
                .html(content)
                .build()
                .unwrap()
                .into(),
        )?)
    }
}

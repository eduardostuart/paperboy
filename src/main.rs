// if the env variables are not defined just panic!
#![allow(clippy::option_env_unwrap)]
use lettre::{message::Mailbox, transport::smtp::authentication::Credentials};
use mailer::MailerConfig;
use paperboy::Paperboy;

use crate::rss::FeedLoader;

mod error;
mod mailer;
mod paperboy;
mod rss;
mod subscriptions;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

const TEMPLATE_FILE: &str = "emails/daily_email.hbs";
const SUBSCRIPTIONS_FILE: &str = "subscriptions.txt";

#[tokio::main]
async fn main() -> Result<()> {
    // Load all RSS/Feed subscriptions from "subscriptions.txt"
    let subscriptions = subscriptions::load_from_file(SUBSCRIPTIONS_FILE);

    // Fetch each subscription item and return all entries from yesterday
    // Empty or invalid items will be ignored
    let result = FeedLoader::new(subscriptions).load().await;

    match result {
        Some(items) => {
            println!("New items: {:#?}", items);

            let mailer_config = MailerConfig {
                from: option_env!("SMTP_FROM").unwrap().to_string(),
                credentials: Credentials::new(
                    option_env!("SMTP_USERNAME").unwrap().to_string(),
                    option_env!("SMTP_PASSWORD").unwrap().to_string(),
                ),
                relay: option_env!("SMTP_RELAY").unwrap().to_string(),
            };

            let to = option_env!("MAIL_TO").unwrap().parse::<Mailbox>().unwrap();

            Paperboy::new(TEMPLATE_FILE, mailer_config)
                .deliver(items, to)
                .await?;
        }
        None => println!("Nothing new for today"),
    }

    Ok(())
}

#[cfg(test)]
pub mod test_util {
    use std::{
        fs::{create_dir_all, File},
        io::Write,
        panic,
    };

    use rand::{distributions::Alphanumeric, Rng};

    pub fn run<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        let result = panic::catch_unwind(|| test());
        assert!(result.is_ok())
    }

    pub fn create_tmp_file(content: &str) -> (String, String) {
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let path = format!(".tmp{}", random);
        let file_path = format!("{}/file.txt", &path);

        create_dir_all(&path).unwrap();

        let file = File::create(&file_path).unwrap();
        write!(&file, "{}", &content).unwrap();

        (String::from(path), file_path.to_string())
    }
}

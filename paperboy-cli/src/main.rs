use std::{fs::File, path::Path, process};

use crate::{cli::Args, deliver::MailConfig};
use clap::StructOpt;
use cli::Commands;
use paperboy::Result;

mod cli;
mod deliver;

use deliver::Deliver;

const SUBSCRIPTIONS_FILE: &str = "subscriptions.txt";

#[tokio::main]
async fn main() -> Result<()> {
    check_for_subscriptions_file();

    let matches = Args::parse();

    match matches.commands {
        // Deliver RSS by email
        Commands::Deliver { email, template } => {
            deliver_rss_by_email(email, template, matches.verbose.is_positive()).await?;
        }
    };

    Ok(())
}

async fn deliver_rss_by_email(email: String, template: String, verbose: bool) -> Result<()> {
    let smtp_port = match option_env!("SMTP_PORT") {
        Some(p) => p.parse::<u16>().unwrap(),
        None => 25,
    };

    let config = MailConfig {
        smtp_from: &get_env_key("SMTP_FROM", "SMTP is not defined"),
        smtp_host: &get_env_key("SMTP_HOST", "SMTP host is not defined"),
        smtp_password: &get_env_key("SMTP_PASSWORD", "SMTP password is not defined"),
        smtp_username: &get_env_key("SMTP_USERNAME", "SMTP username is not defined"),
        smtp_port: &smtp_port,
    };

    let result = Deliver::new(SUBSCRIPTIONS_FILE, &template, config)
        .handle(&email, verbose)
        .await?;

    println!("Result: {:?}", result.message);

    if result.errors.is_some() && verbose {
        println!("Errors: {:#?}", result.errors.unwrap());
    }

    Ok(())
}

fn check_for_subscriptions_file() {
    if !Path::new(SUBSCRIPTIONS_FILE).exists() {
        File::create(SUBSCRIPTIONS_FILE).expect("Error while creating subscriptions file");
    }
}

fn get_env_key(key: &str, error: &str) -> String {
    match std::env::var_os(key) {
        Some(val) => val.into_string().unwrap(),
        None => {
            eprintln!("{}.", error);
            process::exit(1);
        }
    }
}

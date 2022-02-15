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

    let verbose = matches.verbose;

    match matches.commands {
        // Deliver RSS by email
        Commands::Deliver {
            email,
            template_file,
        } => {
            let smtp_from = get_env_key("SMTP_FROM", "SMTP is not defined");
            let smtp_host = get_env_key("SMTP_HOST", "SMTP host is not defined");
            let smtp_password = get_env_key("SMTP_PASSWORD", "SMTP password is not defined");
            let smtp_username = get_env_key("SMTP_USERNAME", "SMTP username is not defined");
            let smtp_port = match option_env!("SMTP_PORT") {
                Some(p) => p.parse::<u16>().unwrap(),
                None => 25,
            };

            let config = MailConfig {
                smtp_from: &smtp_from,
                smtp_host: &smtp_host,
                smtp_password: &smtp_password,
                smtp_username: &smtp_username,
                smtp_port: &smtp_port,
            };

            let template = match template_file {
                Some(v) => v,
                None => "emails/daily_email.hbs".to_string(),
            };

            let wants_verbose = verbose.is_positive();

            let result = Deliver::new(SUBSCRIPTIONS_FILE, &template, config)
                .handle(&email, wants_verbose)
                .await?;

            println!("Result: {:?}", result.message);

            if result.errors.is_some() && wants_verbose {
                println!("Errors: {:?}", result.errors.unwrap());
            }
        }
    };

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

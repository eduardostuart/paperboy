use std::{path::Path, process};

use crate::{cli::Args, deliver::MailConfig};
use clap::StructOpt;
use cli::Commands;
use paperboy::Result;

mod cli;
mod deliver;

use deliver::Deliver;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Args::parse();

    if matches.verbose.log_level().is_some() {
        simple_logger::init_with_level(matches.verbose.log_level().unwrap()).unwrap();
    }

    log::trace!("paperboy cli init");

    match matches.commands {
        // Deliver RSS by email
        Commands::Deliver {
            email,
            subscription_file,
            template,
        } => {
            log::trace!("Deliver command");

            if !Path::new(&subscription_file).is_file() {
                eprint!(
                    "[Error] - Subscription file {} does not exist",
                    subscription_file
                );
                std::process::exit(1);
            }

            deliver_rss_by_email(email, subscription_file, template).await?;
        }
    };

    Ok(())
}

async fn deliver_rss_by_email(
    email: String,
    subscription_file: String,
    template: String,
) -> Result<()> {
    let smtp_port = match option_env!("SMTP_PORT") {
        Some(p) => p.parse::<u16>().unwrap(),
        None => 25,
    };

    let config = MailConfig {
        smtp_from: &get_env_key(
            "SMTP_FROM",
            "[Error] - SMTP_FROM environment variable is not defined",
        ),
        smtp_host: &get_env_key(
            "SMTP_HOST",
            "[Error] - SMTP_HOST environment variable is not defined",
        ),
        smtp_password: &get_env_key(
            "SMTP_PASSWORD",
            "[Error] - SMTP_PASSWORD environment variable is not defined",
        ),
        smtp_username: &get_env_key(
            "SMTP_USERNAME",
            "[Error] - SMTP_USERNAME environment variable is not defined",
        ),
        smtp_port: &smtp_port,
    };

    let result = Deliver::new(&subscription_file, &template, config)
        .handle(&email)
        .await?;

    println!("\nResult: {:?}\n", result.message);

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        eprintln!("({}) Errors:\n{:#?}\n", &errors.len(), &errors);
    }

    Ok(())
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

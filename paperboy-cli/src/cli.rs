use clap::AppSettings;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(version, author = "Eduardo Stuart <e@s.tuart.me>", setting = AppSettings::ArgRequiredElseHelp)]
pub struct Args {
    #[clap(long, short = 'v', parse(from_occurrences), global = true)]
    pub verbose: i8,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Deliver new posts from sites by email
    Deliver {
        /// The recipient
        email: String,
        /// Email template (Using handlebars)
        template: String,
    },
    // TODO
    // #[clap(subcommand)]
    // Subscription(SubscriptionCommands),
}

// #[derive(Subcommand, Debug)]
// pub enum SubscriptionCommands {
//     /// Add new site into subscriptions
//     Add { url: Option<String> },
//     /// Remove site from subscriptions
//     Remove { url: Option<String> },
//     /// List all subscriptions
//     List,
// }

use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(version, author, about, arg_required_else_help(true))]
pub struct Args {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Deliver new posts from sites by email
    Deliver {
        /// The recipient
        email: String,
        /// Subscription file
        subscription_file: String,
        /// Email template as HTML (Using handlebars)
        template_html: String,
        /// Email template as text (Using handlebars)
        template_text: Option<String>,
    },
}

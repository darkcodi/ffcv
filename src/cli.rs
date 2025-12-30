use clap::Parser;

/// CLI arguments for ffcv
#[derive(Parser, Debug)]
#[command(name = "ffcv")]
#[command(about = "View Firefox configuration from the command line")]
pub struct Args {
    /// Firefox profile name (default: "default")
    #[arg(short, long, default_value = "default")]
    pub profile: String,

    /// List all available Firefox profiles
    #[arg(long, conflicts_with = "profile")]
    pub list: bool,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        Parser::parse()
    }
}

use clap::Parser;

/// Dynamic DNS through `CloudFlare`'s API.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// User-supplied IP address.
    #[arg(short, long)]
    pub ip_address: Option<String>,
}

impl Args {
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}

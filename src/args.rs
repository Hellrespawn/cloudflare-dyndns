use clap::Parser;

/// Check public or user-supplied IP address and update A-records at your
/// `CloudFlare` zone using the `CloudFlare` API.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// User-supplied IP address.
    #[arg(short, long)]
    pub ip_address: Option<String>,

    /// Force update of records, even if IP hasn't changed.
    #[arg(short, long, hide = true)]
    pub force: bool,

    /// Don't update records, only show changes.
    #[arg(short, long, visible_short_alias = 'd', visible_alias = "dry_run")]
    pub preview: bool,
}

impl Args {
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}

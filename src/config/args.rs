use camino::Utf8PathBuf;
use clap::Parser;

/// Check public or user-supplied IP address and update A-records at your
/// `CloudFlare` zone using the `CloudFlare` API.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// CLoudflare DNS:Edit token
    #[arg(short, long)]
    pub token: Option<String>,

    #[command(flatten)]
    pub ip_address: IpAddress,

    #[command(flatten)]
    pub cloudflare_zone: CloudflareZone,

    /// Force update of records, even if IP hasn't changed.
    #[arg(short, long, hide = true)]
    pub force: bool,

    /// Don't update records, only show changes.
    #[arg(short, long, visible_short_alias = 'd', visible_alias = "dry_run")]
    pub preview: bool,

    /// Config file
    #[arg(short, long)]
    pub config_file: Option<Utf8PathBuf>,
}

#[derive(Debug, clap::Args)]
#[group(required = false, multiple = false)]
pub struct IpAddress {
    /// User-supplied IP address.
    #[arg(long("ip"))]
    pub address: Option<String>,

    /// User-supplied URL to query public IP-address
    #[arg(long)]
    pub url: Option<String>,
}

#[derive(Debug, clap::Args)]
#[group(required = false, multiple = false)]
pub struct CloudflareZone {
    /// Cloudflare zone ID
    #[arg(long)]
    pub id: Option<String>,

    /// CLoudlfare zone name
    #[arg(long)]
    pub name: Option<String>,
}

impl Args {
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}

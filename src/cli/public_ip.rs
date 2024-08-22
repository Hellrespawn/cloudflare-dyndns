use color_eyre::Result;

use crate::config::Config;
use crate::public_ip::get_public_ip_address;

pub async fn main() -> Result<()> {
    crate::init()?;

    let config = Config::load_config()?;

    let ip_url = config.public_ip_url();

    let public_ip = get_public_ip_address(ip_url).await?;

    println!("Your public IP address is {public_ip}");

    Ok(())
}

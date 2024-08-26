# Dynamic DNS for CloudFlare

Dynamic DNS script for CloudFlare. Checks the public IP and, if changed, update all A-records to point to it.

Includes systemd service and timer.

Requires Rust.

## Installation

Run `cargo install` to install it to your personal `cargo` bin. Make sure it's on your `$PATH`.

If using systemd, copy `service` to `/etc/systemd/cloudflare-dyndns.service` and `timer` to `/etc/systemd/cloudflare-dyndns.timer`.

Alternatively, use the `install.sh`-script.

## Configuration

The script tries to read `/etc/cloudflare-dyndns/cloudflare-dyndns.toml` when running as root or `$HOME/.config/cloudflare-dyndns.toml` when running as user.

The expected format is:

```toml
public_ip_url = "https://example.ip"
cloudflare_token = ""

["example.com"]  # Zone name or id
records = ["example.com", "*", "ftp"]  # A-record names.

["example.nl"]
records = ["example.nl", "*", "mail"]

```

## Usage

```txt
Dynamic DNS for CloudFlare

Usage: cloudflare-dyndns [OPTIONS]

Options:
  -c, --config <CONFIG>          Config file location. Defaults to ~/.config/cloudflare-dyndns.toml or /etc/cloudflare-dyndns/cloudflare-dyndns.toml when running as root
  -i, --ip-address <IP_ADDRESS>  The desired IP address. Defaults to the IP address determined via the `public_ip_url` in the configuration
  -p, --preview                  Shows what would happen, but doesn't change any settings
  -f, --force                    Update records even if the cached IP address hasn't changed
  -h, --help                     Print help
```

### Manual usage

Run `cloudflare-dyndns` to query the public IP address of the current device and point all A-records to it.

You can also run `cloudflare-dyndns -i <IP Address>` to manually specify the IP address.

Consider using the systemd service or adding an entry to your `crontab`.

### crontab

You can add `cloudflare-dyndns` to your crontab.

```crontab
"*/15 * * * *  /opt/cloudflare-dyndns >> /var/log/cloudflare-dyndns.log 2>&1"
```

This will run every 15 minutes, logging the output to a file.

Use the following command or similar to add it to your crontab.

```sh
printf "%s\n%s\n" "$(crontab -l)" "*/15 * * * *  /opt/cloudflare-dyndns >> /var/log/cloudflare-dyndns.log 2>&1" | crontab -
```

### systemd

You can run `cloudflare-dyndns.service` to manually update the IP address once.

```sh
systemctl start cloudflare-dyndns.service
```

You can enable and start `cloudflare-dyndns.timer` to run the script every 15 minutes.

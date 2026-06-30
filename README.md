# ryndns (Ring-a-ding-dyndns)

Dynamic DNS tool that updates your DNS records when your public IP changes. Supports Cloudflare and bunny.net.

Includes systemd service and timer.

Requires Rust.

## Installation

Run `cargo install` to install it to your personal `cargo` bin. Make sure it's on your `$PATH`.

If using systemd, copy `ryndns.service` to `/etc/systemd/system/ryndns.service` and `ryndns.timer` to `/etc/systemd/system/ryndns.timer`.

Alternatively, use the `install.sh` script.

## Configuration

The tool tries to read `/etc/ryndns/ryndns.toml` when running as root or `$HOME/.config/ryndns/ryndns.toml` when running as user.

**Migration note:** If upgrading from `cloudflare-dyndns`, move your config from `~/.config/cloudflare-dyndns/` to `~/.config/ryndns/` and update the format as shown below.

The expected format is:

```toml
public_ip_url = "https://example.ip"

[cloudflare]
token = "your-cloudflare-token"

[[cloudflare.zones]]
name = "example.com"  # Zone name or id
records = ["example.com", "*", "ftp"]  # A-record names

[bunny]
token = "your-bunny-api-key"

[[bunny.zones]]
name = "example.nl"  # Zone name
records = ["example.nl", "*", "mail"]  # A-record names
```

## Usage

```txt
Dynamic DNS tool for Cloudflare and bunny.net

Usage: ryndns [OPTIONS]

Options:
  -c, --config <CONFIG>          Config file location. Defaults to ~/.config/ryndns/ryndns.toml or /etc/ryndns/ryndns.toml when running as root
  -i, --ip-address <IP_ADDRESS>  The desired IP address. Defaults to the IP address determined via the `public_ip_url` in the configuration
  -p, --preview                  Shows what would happen, but doesn't change any settings
  -f, --force                    Update records even if the cached IP address hasn't changed
  -h, --help                     Print help
```

### Manual usage

Run `ryndns` to query the public IP address of the current device and update all configured DNS records to point to it.

You can also run `ryndns -i <IP Address>` to manually specify the IP address.

Consider using the systemd service or adding an entry to your `crontab`.

### crontab

You can add `ryndns` to your crontab.

```crontab
"*/15 * * * *  /opt/ryndns >> /var/log/ryndns.log 2>&1"
```

This will run every 15 minutes, logging the output to a file.

Use the following command or similar to add it to your crontab.

```sh
printf "%s\n%s\n" "$(crontab -l)" "*/15 * * * *  /opt/ryndns >> /var/log/ryndns.log 2>&1" | crontab -
```

### systemd

You can run `ryndns.service` to manually update the IP address once.

```sh
systemctl start ryndns.service
```

You can enable and start `ryndns.timer` to run the tool every 15 minutes.

```sh
systemctl enable --now ryndns.timer
```

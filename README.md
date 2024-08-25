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
Dynamic DNS for CloudFlare.

Usage: cloudflare-dyndns [OPTIONS]

Options:
      --ip <ADDRESS>               User-supplied IP address  # TODO
  -p, --preview                    Don't update records, only show changes [aliases: dry_run] [short aliases: d]  # TODO
  -c, --config-file <CONFIG_FILE>  Custom configuration file  # TODO
  -h, --help                       Print help  # TODO
  -V, --version                    Print version  # TODO
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

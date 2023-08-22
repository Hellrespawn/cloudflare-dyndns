# Dynamic DNS for CloudFlare

Dynamic DNS script for CloudFlare. Checks the public IP and, if changed, update all A-records to point to it.

Includes systemd service and timer.

Requires Rust.

## Installation

### Arch Linux

Run `makepkg` in the `PKGBUILD`-folder to create and install a package.

```sh
makepkg \
  --install \
  --syncdeps \  # Also install dependencies
  --rmdeps  # Remove dependencies after install
```

### Manual installation

Run `cargo install` to install it to your personal `cargo` bin. Make sure it's on your `$PATH`.

If using systemd, copy `service` to `/etc/systemd/cloudflare-dyndns.service` and `timer` to `/etc/systemd/cloudflare-dyndns.timer`.

## Configuration

The script tries to read `/etc/cloudflare-dyndns/cloudflare-dyndns.conf` when running as root or `$HOME/.config/cloudflare-dyndns.conf` when running as user.

The expected format is:

```sh
CLOUDFLARE_TOKEN=<API token>
CLOUDFLARE_ZONE_ID=<Zone ID>
```

## Usage

```txt
Dynamic DNS for CloudFlare.

Usage: cloudflare-dyndns [OPTIONS]

Options:
  -i, --ip-address <IP_ADDRESS>  User-supplied IP address
  -p, --preview                  Don't update records, only show changes [aliases: dry_run] [short aliases: d]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Manual usage

Run `cloudflare-dyndns` to query the public IP address of the current device and point all A-records to it.

You can also run `cloudflare-dyndns -i <IP Address>` to manually specify the IP address.

Consider using the systemd service or adding an entry to your `crontab`.

### crontab

You can add `cloudflare-dyndns` to your crontab.

```crontab
"*/15 * * * *  /usr/bin/cloudflare-dyndns >> /var/log/cloudflare-dyndns.log 2>&1"
```

This will run every 15 minutes, logging the output to a file.

Use the following command or similar to add it to your crontab.

```sh
printf "%s\n%s\n" "$(crontab -l)" "*/15 * * * *  /usr/bin/cloudflare-dyndns >> /var/log/cloudflare-dyndns.log 2>&1" | crontab -
```

### systemd

You can run `cloudflare-dyndns.service` to manually update the IP address once.

```sh
systemctl start cloudflare-dyndns.service
```

You can enable and start `cloudflare-dyndns.timer` to run the script every 15 minutes.

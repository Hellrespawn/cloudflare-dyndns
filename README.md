# Dynamic DNS for CloudFlare

Dynamic DNS script for CloudFlare. Checks the public IP and, if changed, update all A-records to point to it.

Includes systemd service and timer.

## Installation

### Arch Linux

Use `makepkg` to create and install a package.

```sh
makepkg \
  --install \
  --syncdeps  # Also install dependencies
```

### Manual installation

This script relies on `jq` and a POSIX-shell. Copy `cloudflare-dyndns.sh` to a location on your `$PATH`.

If using systemd, copy `service` to `/etc/systemd/cloudflare-dyndns.service` and `timer` to `/etc/systemd/cloudflare-dyndns.timer`.

## Configuration

The script tries to read `/etc/cloudflare-dyndns/cloudflare-dyndns.conf` when running as root or `$HOME/.config/cloudflare-dyndns.conf` when running as user.

The expected format is:

```sh
CLOUDFLARE_TOKEN=<API token>
CLOUDFLARE_ZONE_ID=<Zone ID>
```

## Usage

### Manual usage

Run `cloudflare-dyndns` to query the public IP address of the current device and point all A-records to it.

You can also run `cloudflare-dyndns -i <IP Address>` to manually specify the IP address.

Consider using the systemd service or adding an entry to your `crontab`.

### systemd

You can run `cloudflare-dyndns.service` to manually update the IP address once.

```sh
systemctl start cloudflare-dyndns.service
```

You can enable and start `cloudflare-dyndns.timer` to run the script every 15 minutes.

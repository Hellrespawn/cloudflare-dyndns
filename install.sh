#!/bin/sh

cargo build --release
sudo mkdir -p /opt/cloudflare-dyndns
sudo cp target/release/cloudflare-dyndns /opt/cloudflare-dyndns/
sudo cp cloudflare-dyndns.service /etc/systemd/system/
sudo cp cloudflare-dyndns.timer /etc/systemd/system/
sudo systemctl daemon-reload

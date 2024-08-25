#!/bin/bash

set -o errexit

name="cloudflare-dyndns"
dest="/opt/$name"

cargo build --release
sudo mkdir -p "$dest"
sudo cp target/release/cloudflare-dyndns "$dest/$name"

sudo cp "$name.service" /etc/systemd/system/
sudo cp "$name.timer" /etc/systemd/system/
sudo systemctl daemon-reload

sudo mkdir -p "/etc/$name"
sudo cp $name.example.toml /etc/$name/$name.toml

echo "$name was (re)installed to $dest/"
echo "Edit the configuration at /etc/$name/$name.toml".

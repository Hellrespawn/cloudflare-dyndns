#!/bin/bash

set -o errexit

name="cloudflare-dyndns"
dest="/opt/$name"

sudo mv "/etc/$name/$name.toml" "/etc/$name/$name.toml.save"
sudo rm "/etc/systemd/system/$name.service"
sudo rm "/etc/systemd/system/$name.timer"
sudo rm -r "$dest"

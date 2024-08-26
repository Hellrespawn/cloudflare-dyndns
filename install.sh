#!/bin/bash

set -o errexit

name="cloudflare-dyndns"
dest="/opt/$name"

action="install"
force=

while getopts "fu" flag; do
    case $flag in
    f)
        force=1
        ;;
    u)
        action="uninstall"
        ;;
    *)
        echo "Usage: $0 [-u] [-f]"
        exit 1
        ;;
    esac
done

install() {
    cargo build --release

    sudo mkdir -p "$dest"
    sudo cp target/release/cloudflare-dyndns "$dest/$name"

    echo "Installed $name to $dest"

    sudo cp "$name.service" /etc/systemd/system/
    sudo cp "$name.timer" /etc/systemd/system/

    echo "Installed systemd service and timer"

    sudo mkdir -p "/etc/$name"
    sudo cp $name.example.toml /etc/$name/$name.toml

    echo "Edit the configuration at /etc/$name/$name.toml"

}

uninstall() {
    config="/etc/$name/$name.toml"
    backup="/etc/$name/$name.toml.save"

    if [ -z "$force" ] && [ -f "$config" ]; then
        sudo mv "$config" "$backup"
        echo "Backed up configuration to $backup"
    fi

    if [ -n "$force" ]; then
        sudo rm -rf /etc/$name/
        echo "Removed configuration from /etc/$name"
    fi

    sudo rm -f "/etc/systemd/system/$name.service"
    sudo rm -f "/etc/systemd/system/$name.timer"

    echo "Removed systemd service and timer"

    sudo rm -rf "$dest"

    echo "Removed $dest"
}

if [ "$action" == "install" ]; then
    install
elif [ "$action" == "uninstall" ]; then
    uninstall
fi

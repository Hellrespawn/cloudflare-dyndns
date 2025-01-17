#!/bin/bash

set -o errexit

name="cloudflare-dyndns"

bin_dir="/opt/$name"
config_path="/etc/$name/$name.toml"

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

    sudo sh -c "install -Dm755 target/release/cloudflare-dyndns $bin_dir/$name"

    echo "Installed $name to $bin_dir/$name..."

    sudo sh -c "install -Dm644 $name.timer -t /etc/systemd/system/"
    sudo sh -c "install -Dm644 $name.service -t /etc/systemd/system/"

    echo "Installed systemd service and timer..."

    if [ -f "$config_path" ] && [ -z "$force" ]; then
        echo "Configuration file exists at $config_path..."
    else
        if [ -n "$force" ]; then
            echo "Overwriting existing configuration because of -f..."
        fi

        sudo sh -c "install -Dm644 $name.example.toml $config_path"

        echo "Installed example configuration to $config_path..."
    fi

}

uninstall() {
    backup="/etc/$name/$name.toml.save"

    if [ -f "$config_path" ] && [ -z "$force" ]; then
        sudo mv "$config_path" "$backup"
        echo "Backed up configuration to $backup..."
    fi

    if [ -n "$force" ]; then
        sudo rm -rf /etc/$name/
        echo "Removed configuration from /etc/$name because of -f..."
    fi

    sudo rm -f "/etc/systemd/system/$name.service"
    sudo rm -f "/etc/systemd/system/$name.timer"

    echo "Removed systemd service and timer..."

    sudo rm -rf "$bin_dir"

    echo "Removed $bin_dir..."
}

if [ "$action" == "install" ]; then
    install
elif [ "$action" == "uninstall" ]; then
    uninstall
fi

echo "Done."

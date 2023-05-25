#!/usr/bin/env sh

set -o errexit
set -o nounset
if [ "${TRACE-0}" = "1" ]; then set -o xtrace; fi

CONFIG_FILE="cloudflare-dyndns.conf"
LAST_IP_FILE="last-ip"

SYSTEM_CONFIG_DIR="/etc/cloudflare-dyndns"
USER_CONFIG_DIR="${HOME-"/dev/null"}/.config"

API_URL="https://api.cloudflare.com/client/v4"

exit_with_error() {
    printf "Error: %s\n" "$@"
    usage
    exit 1
}

usage() {
    echo "Usage: $0 [-h | -i ip_address]"
}

# Silence curl progess bars
curl() {
    command curl -s "$@"
}

# Wrapper function for curl posts. Adds Authorization and Content-Type header.
# Optionally adds JSON data.
#
# $1: HTTP method
# $2: URL
# $3: JSON Data (optional)
curl_wrapper() {
    command="curl -X '$1' '$API_URL$2' -H 'Content-Type: application/json' -H 'Authorization: Bearer $CLOUDFLARE_TOKEN'"

    if [ -n "${3-}" ]; then command="$command -d '$3'"; fi

    eval "$command"
}

init_environment() {
    if ! command -v jq >/dev/null; then exit_with_error "jq is required."; fi

    # shellcheck source=/dev/null
    if [ -f "$SYSTEM_CONFIG_DIR/$CONFIG_FILE" ]; then . "$SYSTEM_CONFIG_DIR/$CONFIG_FILE"; fi
    # shellcheck source=/dev/null
    if [ -f "$USER_CONFIG_DIR/$CONFIG_FILE" ]; then . "$USER_CONFIG_DIR/$CONFIG_FILE"; fi
    if [ -z "${CLOUDFLARE_TOKEN-}" ]; then exit_with_error "CLOUDFLARE_TOKEN is not set."; fi
    if [ -z "${CLOUDFLARE_ZONE_ID-}" ]; then exit_with_error "CLOUDFLARE_ZONE_ID is not set."; fi
}

handle_arguments() {
    # ':'-prefix indicates we handle errors ourselves.
    #
    # ':'-suffix in 'i:' indicates it should receive an argument. It is
    # required for $OPTARG to be set.
    while getopts ': i:h' opt; do
        case $opt in
        i)
            if [ -z "${OPTARG-}" ]; then
                exit_with_error "'-i' requires an IP address as argument."
            fi

            NEW_IP=$OPTARG
            ;;
        h)
            usage
            exit 0
            ;;
        *)
            exit_with_error "Invalid option '-$OPTARG'"
            ;;
        esac
    done

    shift $((OPTIND - 1))
}

# Returns true if the ip is changed.
#
# $1: new ip
is_public_ip_changed() {
    last_ip=$(get_last_ip)

    if [ -n "$last_ip" ]; then
        if [ "$last_ip" != "$1" ]; then
            echo "IP address updated: '$last_ip' => '$1'"
            return 0
        else

            echo "IP address unchanged: '$last_ip'"
            return 1
        fi
    else
        echo "IP address: '$1'"
        return 0
    fi
}

get_last_ip() {
    ip_file=$(get_last_ip_file)

    if [ -f "$ip_file" ]; then
        cat "$ip_file"
    fi
}

get_last_ip_file() {
    if [ "$(id -u)" = "0" ]; then
        echo "$SYSTEM_CONFIG_DIR/$LAST_IP_FILE"
    else
        echo "$USER_CONFIG_DIR/$LAST_IP_FILE"
    fi
}

save_last_ip() {
    ip_file=$(get_last_ip_file)

    echo "$1" >"$ip_file"
}

# Gets the public ip of the server.
get_public_ip() {
    TEMP_FILE=$(mktemp)
    status=$(curl -o "$TEMP_FILE" -w "%{response_code}" 'http://ipecho.net/plain')

    if [ "$status" -ne "200" ]; then
        exit_with_error "Received status $status while getting public IP.\nSee $TEMP_FILE for response."
    fi

    public_ip=$(cat "$TEMP_FILE")

    rm "$TEMP_FILE"

    echo "$public_ip"
}

# Gets all DNS records for $CLOUDFLARE_DOMAIN
get_dns_records() {
    curl_wrapper GET "/zones/$CLOUDFLARE_ZONE_ID/dns_records"
}

# $1: API Response, JSON Object
check_api_error() {
    result=$(echo "$1" | jq '.success')

    if [ "$result" != "true" ]; then
        exit_with_error "$($1 | jq '.errors')"
    fi
}

main() {
    init_environment

    handle_arguments "$@"

    new_ip=${NEW_IP-$(get_public_ip)}

    if is_public_ip_changed "$new_ip"; then
        response=$(get_dns_records)
        check_api_error "$response"

        echo "Retrieved DNS-records."

        a_records=$(echo "$response" | jq -c '.result[] | select(.type == "A")')

        for record in $a_records; do
            id=$(echo "$record" | jq -r '.id')

            response=$(curl_wrapper PATCH "/zones/$CLOUDFLARE_ZONE_ID/dns_records/$id" "{ \"content\": \"$new_ip\" }")
            check_api_error "$response"

            printf "Updated '%s' record.\n" "$(echo "$record" | jq -r '.name')"
        done

        save_last_ip "$new_ip"

        echo "Updated IP address in cache".
    fi
}

main "$@"

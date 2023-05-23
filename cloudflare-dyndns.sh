#!/bin/sh

if ! command -v jq >/dev/null; then
    echo "jq is required."
    exit 1
fi

CLOUDFLARE_CONF="${CLOUDFLARE_CONF:-"./.env"}"
CLOUDFLARE_IP_FILE="${CLOUDFLARE_IP_FILE:-"./last-ip"}"

if [ ! -f "$CLOUDFLARE_CONF" ]; then
    echo "Configuration file '$CLOUDFLARE_CONF' doesn't exist."
    exit 1
fi

set -e

. "$CLOUDFLARE_CONF"

if [ -z "$CLOUDFLARE_TOKEN" ]; then
    echo "CLOUDFLARE_TOKEN is not set."
    exit 1
fi

if [ -z "$CLOUDFLARE_ZONE_ID" ]; then
    echo "CLOUDFLARE_ZONE_ID is not set."
    exit 1
fi

API_URL="https://api.cloudflare.com/client/v4"

PUBLIC_IP=

# Silence curl progess bars
curl() {
    command curl -s "$@"
}

# Wrapper function for curl posts. Sets default headers.
#
# $1: HTTP method
# $2: URL
# $3: JSON Data (optional)
curl_json() {
    command="curl -X '$1' '$API_URL$2' -H 'Content-Type: application/json' -H 'Authorization: Bearer $CLOUDFLARE_TOKEN'"

    if [ -n "$3" ]; then
        command="$command -d '$3'"
    fi

    eval "$command"
}

# Returns true if the ip is changed. Sets global $PUBLIC_IP
is_ip_changed() {
    PUBLIC_IP=$(get_public_ip)

    if [ -f "$CLOUDFLARE_IP_FILE" ]; then
        last_ip=$(cat "$CLOUDFLARE_IP_FILE")

        if [ "$last_ip" != "$PUBLIC_IP" ]; then
            echo "IP address updated: '$last_ip' => '$PUBLIC_IP'"
            return 0
        else

            echo "IP address unchanged: '$PUBLIC_IP'"
            return 1
        fi
    else
        echo "IP address: '$PUBLIC_IP'"
        return 0
    fi
}

# Gets the public ip of the server.
get_public_ip() {
    TEMP_FILE=$(mktemp)
    status=$(curl -o "$TEMP_FILE" -w "%{response_code}" 'http://ipecho.net/plain')

    if [ "$status" -ne "200" ]; then
        echo "Received status $status while getting public IP."
        echo "See $TEMP_FILE for response."
        exit 1
    fi

    public_ip=$(cat "$TEMP_FILE")

    rm "$TEMP_FILE"

    echo "$public_ip"
}

# Gets all DNS records for $CLOUDFLARE_DOMAIN
get_dns_records() {
    curl_json GET "/zones/$CLOUDFLARE_ZONE_ID/dns_records"
}

# $1: API Response, JSON Object
check_api_error() {
    result=$(echo "$1" | jq '.success')

    if [ "$result" != "true" ]; then
        echo "An error occurred."
        echo "$1" | jq '.errors'
        exit 1
    fi
}

update_last_ip() {
    echo "$PUBLIC_IP" >"$CLOUDFLARE_IP_FILE"
}

main() {
    new_ip=

    if [ -n "$1" ]; then
        new_ip="$1"
    elif is_ip_changed; then
        new_ip="$PUBLIC_IP"
    fi

    if [ -n "$new_ip" ]; then
        response=$(get_dns_records)
        check_api_error "$response"

        echo "Retrieved DNS-records."

        a_records=$(echo "$response" | jq -c '.result[] | select(.type == "A")')

        # identifiers=$(extract_identifiers "$records")

        for record in $a_records; do
            id=$(echo "$record" | jq -r '.id')

            response=$(curl_json PATCH "/zones/$CLOUDFLARE_ZONE_ID/dns_records/$id" "{ \"content\": \"$new_ip\" }")
            check_api_error "$response"

            printf "Updated '%s' record.\n" "$(echo "$record" | jq -r '.name')"
        done

        update_last_ip

        echo "Updated IP address in cache".
    fi
}

main "$@"

#!/bin/sh

if ! command -v jq > /dev/null ; then
    echo "jq is required."
    exit 1
fi

if [ ! -f "./.env" ];then
    echo ".env-file doesn't exist."
    exit 1
fi

# shellcheck source=.env
. ./.env

TOKEN_URL="https://api.vimexx.nl/auth/token"
API_URL="https://api.vimexx.nl/api/v1/whmcs/domain/dns"

GET_TOKEN_JSON="{ \
    \"grant_type\": \"password\", \
    \"client_id\": \"$VIMEXX_CLIENT_ID\", \
    \"client_secret\": \"$VIMEXX_CLIENT_SECRET\", \
    \"username\": \"$VIMEXX_USERNAME\", \
    \"password\": \"$VIMEXX_PASSWORD\", \
    \"scope\": \"whmcs-access\" \
}"

GET_DNS_JSON="{ \
    \"body\": { \
        \"sld\":\"$DOMAIN_NAME\", \
        \"tld\":\"nl\" \
    }, \
    \"version\": \"7.7.1-release.1\" \
}"

UPDATE_DNS_JSON_CLOSE="}, \
\"version\": \"7.7.1-release.1\" \
}"

UPDATE_DNS_JSON_OPEN="{ \
\"body\": { \
\"sld\":\"$DOMAIN_NAME\", \
\"tld\":\"nl\" \, \
\"dns_records\":"

PUBLIC_IP=
BEARER_TOKEN=

set -e

# Silence curl progess bars
curl() {
    command curl -s "$@"
}

# Disable jq colors
jq() {
    command jq -M "$@"
}

# Wrapper function for curl posts. Sets default headers.
#
# $1: HTTP method
# $2: URL
# $3: JSON string
# $4: Bearer Token (optional)
curl_json() {
    command="curl -X $1 '$2' -H 'Content-Type: application/json' -d '$3'"

    if [ -n "$4" ]; then
        command="$command -H 'Authorization: Bearer $4'"
    fi

    eval "$command"
}

# Returns true if the ip is changed. Sets global $PUBLIC_IP
is_ip_changed() {
    PUBLIC_IP=$(get_public_ip)

    if [ -f "$IP_FILE" ]; then
        last_ip=$(cat "$IP_FILE")

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
get_public_ip()
{
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

# Gets access token if one doesn't exist or is expired.
# Sets global $BEARER_TOKEN.
refresh_token() {
    if [ ! -f "./token.json" ]; then
        echo "Creating token file '$TOKEN_FILE'"

        update_token
    elif is_token_expired; then
        echo "Token expired, updating '$TOKEN_FILE'"

        update_token
    fi

    BEARER_TOKEN=$(jq -r '.access_token' "$TOKEN_FILE")
}

# Checks if token is expired. Defaults to half of expiration.
is_token_expired() {
    expiration=$(jq '.expires_in' token.json)

    # seconds since epoch
    current_time=$(date +%s)
    mtime=$(stat -c '%Y' "$TOKEN_FILE")


    if [ $((current_time - mtime > expiration / 2)) ]; then
        return 1
    fi

    return 0
}

# Get and update access token
update_token() {
    token=$(curl_json POST "$TOKEN_URL" "$GET_TOKEN_JSON")

    echo "$token" > "$TOKEN_FILE"
}

# Gets all DNS records for $DOMAIN
get_dns_records() {
    response=$(curl_json POST "$API_URL" "$GET_DNS_JSON" "$BEARER_TOKEN")
    check_api_error "$response"
    echo "response" | jq '.data.dns_records'
}

# $1: API Response, JSON Object
check_api_error() {
    result=$(echo "$1" | jq '.result')

    if [ "$result" != "true" ]; then
        echo "An error occurred while gettings DNS records."
        echo "message: $(echo "$1" | jq '.message')"
        exit 1
    fi
}

# Transforms DNS records by updating A records, deleting AAAA records,
# removing null-items and settings the ttl.
#
# [  # Wrap output in array
# .[]  # Select all array items
# | select(.type != "AAAA")  # Select only items that aren't AAAA-records
# | .content = if .type == "A" then "$PUBLIC_IP" else .content end  # Only change A-records
# | del(.prio|nulls)  # Delete every .prio that is null
# | .ttl = 300  # Add ttl of 300
# ]
#
# $1: JSON array of DNS records
# $2: New IP address
transform_dns_records() {
    echo "$1" | jq "[.[] | select(.type != \"AAAA\") | .content = if .type == \"A\" then \"$2\" else .content end | del(.prio|nulls) | .ttl = 300]"
}

# $1: JSON array with transformed values
update_dns_records() {
    body="$UPDATE_DNS_JSON_OPEN $1 $UPDATE_DNS_JSON_CLOSE"

    echo "$body" | command jq '.'

    # response=$(curl_json POST "$API_URL" "$body" "$BEARER_TOKEN")
    # check_api_error "$response"
}

update_last_ip() {
    echo "$PUBLIC_IP" > "$IP_FILE"
}

new_ip=

if [ -n "$!" ]; then
    new_ip="$!"
elif is_ip_changed; then
    new_ip="$PUBLIC_IP"
fi

if [ -n "$new_ip" ]; then
    refresh_token

    records=$(get_dns_records)

    echo "Retrieved DNS records"

    new_ip=
    if [ -n "$1" ]; then
        new_ip="$!"
    else
        new_ip="$PUBLIC_IP"
    fi

    body=$(transform_dns_records "$records" "$new_ip")

    update_dns_records "$body"

    echo "Updated DNS records".

    update_last_ip

    echo "Updated IP address in cache".
fi

echo "Done"

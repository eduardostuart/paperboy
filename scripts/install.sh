#!/bin/sh
#
# Download latest version of paperboy-cli
#

set -e


info(){
    echo '[INFO] ' "$@"
}

error() {
    echo '[ERROR] ' "$@" >&2
    exit 1
}


if [ ! -x "$(which curl)" ] ; then
    error "curl is not installed"
fi

if [ ! -x "$(which jq)" ] ; then
    error "jq is not installed"
fi


REPOSITORY_URL="https://api.github.com/repos/eduardostuart/paperboy"

case "$(uname -s)" in
    Darwin)
        TARGET="apple-darwin.zip"
        ;;
    *)
        TARGET="linux-musl.tar.gz"
        ;;
esac


info "Fetching latest release from $REPOSITORY_URL/releases/latest..."

DOWNLOAD_URL=$(curl -s $REPOSITORY_URL/releases/latest | jq -M -r '.assets[] | select(.browser_download_url | endswith('\""$TARGET"\"')) | .browser_download_url')


if [[ ! -z "$DOWNLOAD_URL" ]]
then
    info $DOWNLOAD_URL
    FILE="o.tar.gz"
    # download and extract
    curl -sL -o $FILE $DOWNLOAD_URL 
    tar -xzf $FILE 
    rm -rf $FILE
    # set permissions
    chmod +x ./paperboy

    info "Done"
else
    error "The latest release download url could not be found"
fi
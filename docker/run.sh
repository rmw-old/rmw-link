#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex
docker run -it --rm rmw-link-build -v $DIR/../rust:/root/rust /bin/zsh -c "cd /root/rust"

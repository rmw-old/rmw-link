#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex
docker run -it --rm \
  -v $DIR/../rust:/root/rust \
  -v $DIR/cache/cargo/git:/opt/cargo/git \
  -v $DIR/cache/cargo/registry:/opt/cargo/registry \
  rmw-link-build \
  /bin/sh -c "cd /root/rust && /bin/zsh"

#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex
docker run -it --rm \
  -v $DIR/../rust:/root/rust\
  rmw-link-build \
  /bin/sh -c "cd /root/rust && /bin/zsh"

#!/usr/bin/env zsh

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

docker run --name rmw-link-build \
  -v $DIR/../rust:/root/rust \
  -v $DIR/cache/rust/git:/opt/rust/git \
  -v $DIR/cache/rust/registry:/opt/rust/registry \
  rmw-link-build \
  /bin/zsh -c "cd /root/rust && ./build.xsh"

docker cp rmw-link-build:/root/rust/target/x86_64-unknown-linux-gnu/release/rmw rmw

chmod +x rmw

tar caf rmw-linux-x64_64.tar.xz rmw

#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

docker run --name rmw-link-build \
  -v $DIR/../rust:/root/rust \
  -v $DIR/cache/rust/git:/opt/rust/git \
  -v $DIR/cache/rust/registry:/opt/rust/registry \
  rmwl/build \
  /bin/zsh -c "cd /root/rust && rustup default nightly && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu && ./build.xsh"

docker cp rmw-link-build:/root/rust/target/x86_64-unknown-linux-gnu/release/rmw rmw

chmod +x rmw

tar caf rmw-x86_64-linux-gun.tar.xz rmw

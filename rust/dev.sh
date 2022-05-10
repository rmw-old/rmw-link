#!/usr/bin/env bash

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

. ./sh/pid.sh

set -ex

if ! hash watchexec 2>/dev/null; then
cargo install watchexec-cli
fi

project=rmw

#if [ ! -d target ] ; then
cargo build -p $project
#fi

RUST_BACKTRACE=1 watchexec \
  --shell=none -w . \
  -c -r --exts rs,toml \
  --ignore target/ \
  -- cargo +nightly run -p $project

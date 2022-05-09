#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

docker buildx build -t rmw-link-build .
./build.sh

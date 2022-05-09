#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

plugin=$HOME/.docker/cli-plugins

if [ -e "$plugin/docker-buildx" ]; then
mkdir -p $plugin
buildx=$plugin/docker-buildx
wget https://github.com/docker/buildx/releases/download/v0.8.2/buildx-v0.8.2.linux-amd64 -O $buildx
chmod +x $buildx
fi

docker buildx build -t rmwl/build .
docker push rmwl/build

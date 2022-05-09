#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

git pull

VER=`cat version`

tag(){
git commit -m "v$VER" || true
git tag -d v$VER | true
git tag v$VER
git push origin v$VER -f
git push -f
}

tag

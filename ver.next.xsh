#!/usr/bin/env xonsh

p"~/.xonshrc".exists() && source ~/.xonshrc

from os.path import dirname,abspath

PWD = dirname(abspath(__file__))
cd @(PWD)
git pull

VER = $(cat version)
VER = list(map(int,VER.split('.')))
VER[-1] += 1
VER = '.'.join(tuple(map(str,VER)))
echo @(VER) > version

$VER=VER

git commit -m "v$VER" || true
git tag -d v$VER | true
git tag v$VER
git push origin v$VER -f
git push -f

#!/usr/bin/env xonsh

p"~/.xonshrc".exists() && source ~/.xonshrc

from os.path import dirname,abspath

PWD = dirname(abspath(__file__))
cd @(PWD)

VER = $(cat ver)
VER = list(map(int,VER.split('.')))
VER[-1] += 1
VER = '.'.join(tuple(map(str,VER)))
echo @(VER) > ver

git add -u

$VER=VER

git commit -m "v$VER" || true

git pull

git tag -d v$VER | true
git tag v$VER
git push origin v$VER -f
git push -f

#!/usr/bin/env xonsh

from os.path import dirname,abspath
import platform
PWD = dirname(abspath(__file__))
cd @(PWD)

p".xonshrc".exists() && source .xonshrc

system = platform.system().lower()
if system == 'darwin':
  system = f'apple-{system}'
elif system == 'linux':
  system = 'unknown-linux-musl'
# system = 'unknown-linux-gnu'

TARGET=f'{platform.machine()}-{system}'

NAME="rmw"

$RUSTFLAGS="-C target-feature=+crt-static"

cargo build \
--release \
-Z build-std=std,panic_abort \
-Z build-std-features=panic_immediate_abort \
-p @(NAME) \
--target @(TARGET)

out=f"target/{TARGET}/release/{NAME}"
strip @(out)

./sh/upx.sh

upx --best --lzma @(out)

print(out)

export CARGO_HOME=/opt/rust
export RUSTUP_HOME=/opt/rust
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

CARGO_ENV=$CARGO_HOME/env

if [ -e "$CARGO_ENV" ]; then
source $CARGO_ENV
fi

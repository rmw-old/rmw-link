#!/usr/bin/env bash

DIR=$(cd "$(dirname "$0")"; pwd)
set -ex
cd $DIR

case $(uname | tr '[:upper:]' '[:lower:]') in
  linux*)
    export OS=linux
    ;;
  darwin*)
    export OS=macos
    ;;
  msys*)
    export OS=windows
    ;;
  *)
    export OS=undefined && exit 1
    ;;
esac

flutter run -d $OS

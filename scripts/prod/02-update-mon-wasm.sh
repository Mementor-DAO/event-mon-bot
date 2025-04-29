#!/bin/bash

set -e

export MONITOR_RELEASE_DIR=./target/wasm32-unknown-unknown/release

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts/dev" ]; then
  cd ../..
fi

. .env

./scripts/build-monitor.sh
monitor_wasm=$(od -t x1 -v -w1048576 -A n $MONITOR_RELEASE_DIR/monitor.gz | sed "s/ /\\\/g")

dfx canister call bot update_monitors -v --ic --identity deployer --argument-file <(echo "(
    record {
      wasm = blob \"$monitor_wasm\";
    }
)")

popd
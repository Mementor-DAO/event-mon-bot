#!/bin/bash

set -e

export RELEASE_DIR=./target/wasm32-wasip1/release
export MONITOR_RELEASE_DIR=./target/wasm32-unknown-unknown/release

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts/dev" ]; then
  cd ../..
fi

. .env

./scripts/build-monitor.sh
monitor_wasm=$(od -t x1 -v -w1048576 -A n $MONITOR_RELEASE_DIR/monitor.gz | sed "s/ /\\\/g")

dfx canister create bot >/dev/null

ADMIN_PRINCIPAL=$(dfx identity get-principal --identity deployer)

dfx deploy bot -v --ic --identity deployer --with-cycles 1000000000000 --argument-file <(echo "(
    record {
      oc_public_key = \"$OC_PUBLIC_KEY_PROD\";
      administrator = principal \"$ADMIN_PRINCIPAL\";
      monitor_wasm = blob \"$monitor_wasm\";
    }
)")

popd
#!/bin/bash

set -e

export RELEASE_DIR=./target/wasm32-wasip1/release

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts/dev" ]; then
  cd ../..
fi

. .env

dfx canister create bot >/dev/null
dfx canister create monitor >/dev/null

ADMIN_PRINCIPAL=$(dfx identity get-principal)
BOT_CANISTER_ID=$(dfx canister id bot)

dfx deploy monitor -v --identity default --with-cycles 10000000000000 --argument "(
    record {
      bot_canister_id = \"$BOT_CANISTER_ID\";
      administrator = principal \"$ADMIN_PRINCIPAL\";
    }
)"

popd
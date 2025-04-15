#!/bin/bash

set -e

export RELEASE_DIR=./target/wasm32-wasip1/release

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts/dev" ]; then
  cd ../..
fi

. .env

dfx canister create bot >/dev/null

ADMIN_PRINCIPAL=$(dfx identity get-principal)

dfx deploy bot -v --identity default --with-cycles 10000000000000 --argument "(
    record {
      oc_public_key = \"$OC_PUBLIC_KEY\";
      administrator = principal \"$ADMIN_PRINCIPAL\";
    }
)"

popd
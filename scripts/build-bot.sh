#!/bin/bash

set -e

export RELEASE_DIR=./target/wasm32-wasip1/release

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts" ]; then
  cd ..
fi

NAME=bot

cargo build --package $NAME --release --target wasm32-wasip1
wasi2ic $RELEASE_DIR/$NAME.wasm $RELEASE_DIR/$NAME.wasm
candid-extractor $RELEASE_DIR/$NAME.wasm >./src/$NAME.did
#ic-wasm $RELEASE_DIR/$NAME.wasm -o $RELEASE_DIR/$NAME.wasm shrink
#ic-wasm $RELEASE_DIR/$NAME.wasm -o $RELEASE_DIR/$NAME.wasm optimize Oz
ic-wasm $RELEASE_DIR/$NAME.wasm -o $RELEASE_DIR/$NAME.wasm metadata candid:service -f ./src/$NAME.did -v public
gzip --best -c $RELEASE_DIR/$NAME.wasm >$RELEASE_DIR/$NAME.gz

popd
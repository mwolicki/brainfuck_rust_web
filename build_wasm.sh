#!/bin/bash

export EMMAKEN_CFLAGS="--js-library docs/external.js" 
#export EMMAKEN_CFLAGS="-s \"BINARYEN_METHOD='native-wasm'\""
#rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 
cargo +nightly test
cargo +nightly build --target=wasm32-unknown-emscripten --release
rm docs/brainfuck_webassembly-*.wasm docs/brainfuck_webassembly-*.js
cp target/wasm32-unknown-emscripten/release/brainfuck_webassembly.js docs
cp target/wasm32-unknown-emscripten/release/deps/*.wasm docs
cp target/wasm32-unknown-emscripten/release/deps/*.js docs
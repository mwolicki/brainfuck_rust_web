#!/bin/bash

export EMMAKEN_CFLAGS="--js-library external.js" 
#-s \"BINARYEN_METHOD='native-wasm,asmjs'\"
rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 

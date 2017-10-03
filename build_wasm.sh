#!/bin/bash

export EMMAKEN_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0 --js-library external.js" 
rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 

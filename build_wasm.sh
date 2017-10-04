#!/bin/bash

export EMMAKEN_CFLAGS="--js-library external.js" 
rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 

#!/bin/bash

set -e

export CC=clang
export CXX=clang++
cargo leptos build --release

mkdir -p flashcard-app/target

# Executable file
cp target/release/flashcard-app flashcard-app/
# Static files
cp -r target/site flashcard-app/target/

tar -cf flashcard-app.tar.gz flashcard-app

# Clean up temporary directory
rm -rf flashcard-app
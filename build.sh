#!/bin/bash

cargo leptos build --release

mkdir -p temp_package/target

# Executable file
cp target/release/flashcard-app temp_package/
# Static files
cp -r target/site temp_package/target/

tar -cf flashcard-app.tar.gz -C temp_package .

# Clean up temporary directory
rm -rf temp_package
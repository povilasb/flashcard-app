#!/bin/bash

set -e

cargo leptos build --release

mkdir -p flashcard-app/target
mkdir -p flashcard-app/lib

# Executable file
cp target/release/flashcard-app flashcard-app/
# Static files
cp -r target/site flashcard-app/target/

# Get all nix store libraries that the binary depends on
nix_libs=$(otool -L target/release/flashcard-app | grep "/nix/store" | awk '{print $1}' | sort -u)

echo "Found nix store libraries:"
echo "$nix_libs"

# Copy each library to flashcard-app/lib
for lib in $nix_libs; do
    if [ -f "$lib" ]; then
        echo "Copying $lib to flashcard-app/lib/"
        cp "$lib" flashcard-app/lib/

        # Get just the filename
        lib_name=$(basename "$lib")

        # Update the binary to use the local library
        echo "Updating binary to use ./lib/$lib_name"
        install_name_tool -change "$lib" "./lib/$lib_name" flashcard-app/flashcard-app
    else
        echo "Warning: Library $lib not found"
    fi
done

# Add rpath to look in ./lib
install_name_tool -add_rpath ./lib flashcard-app/flashcard-app

echo "Final library dependencies:"
otool -L flashcard-app/flashcard-app

tar -cf flashcard-app.tar.gz flashcard-app

# Clean up temporary directory
rm -rf flashcard-app
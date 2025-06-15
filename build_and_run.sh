#!/bin/bash
# build_and_run.sh
# Purpose: Build and run the ocultum binary locally for development or production on the host machine.
# Usage: ./build_and_run.sh
# Prerequisites: Rust toolchain installed, run from the project root directory.
set -e

echo "=== git pull ==="
git pull

echo "=== build ==="
cargo build --release

echo "=== run locally ==="
"$(dirname "$0")/target/release/ocultum"

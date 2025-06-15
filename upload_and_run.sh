#!/bin/bash
set -e

echo "=== git pull ==="
git pull

echo "=== build ==="
cargo build --release

echo "=== run locally ==="
./target/release/ocultum

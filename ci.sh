#!/bin/bash
# ehatrom CI script

echo "🚀 Running ehatrom CI checks..."

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Running local CI..."
    cd ehatrom
    cargo +nightly fmt -- --check && \
    cargo +nightly clippy --workspace --all-targets -- -D warnings && \
    cargo build --workspace --all-targets && \
    cargo test --workspace --all-targets
    exit $?
fi

# Check if Docker image exists
if ! docker images | grep -q "ehatrom-ci"; then
    echo "🔨 Building Docker image..."
    docker build -t ehatrom-ci ./ehatrom
fi

echo "🐳 Running CI in Docker..."
docker run --rm -v "$(pwd)/ehatrom":/ehatrom -w /ehatrom ehatrom-ci \
    sh -c "cargo fmt -- --check && echo '✅ Format OK' && cargo clippy --workspace --all-targets -- -D warnings && echo '✅ Clippy OK' && cargo test && echo '✅ Tests OK'"

echo "🎉 CI completed successfully!"

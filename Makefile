# Build Docker image for CI
ehatrom_build_ci:
	docker build -t ehatrom-ci ./ehatrom

# Test simple command
test:
	echo "Make test works"

# Run CI checks in Docker (simple version)
ehatrom_ci:
	@echo "Running: docker run --rm -v $(pwd)/ehatrom:/ehatrom -w /ehatrom ehatrom-ci cargo test"
	docker run --rm -v "$$(pwd)/ehatrom":/ehatrom -w /ehatrom ehatrom-ci cargo test

# Run full CI checks in Docker (verbose)
ehatrom_ci_full:
	docker run --rm -v "$$(pwd)/ehatrom":/ehatrom -w /ehatrom ehatrom-ci \
		bash -c "cargo +nightly fmt -- --check && cargo +nightly clippy --workspace --all-targets -- -D warnings && cargo build --workspace --all-targets --verbose && cargo test --workspace --all-targets --verbose"

# Run CI checks locally (without Docker)
ehatrom_ci_local:
	cd ehatrom && \
	cargo +nightly fmt -- --check && \
	cargo +nightly clippy --workspace --all-targets -- -D warnings && \
	cargo build --workspace --all-targets && \
	cargo test --workspace --all-targets

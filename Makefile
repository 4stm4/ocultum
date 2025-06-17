ocultum_ci_local:
	rustup install nightly; \
	rustup default nightly; \
	rustup component add --toolchain nightly rustfmt; \
	rustup component add --toolchain nightly clippy; \
	cargo +nightly fmt -- --check; \
	cargo +nightly clippy --workspace --all-targets -- -D warnings; \
	cargo build --workspace --all-targets --verbose; \
	cargo test --workspace --all-targets --verbose; \
	cargo build --release; \
	cargo doc --no-deps

ocultum_ci:
	docker run --rm -it \
		-v "$(PWD)":/ocultum \
		-w /ocultum \
		ocultum-ci \
		bash -c "cargo +nightly fmt -- --check && \
		         cargo +nightly clippy --workspace --all-targets -- -D warnings && \
		         cargo build --workspace --all-targets --verbose && \
		         cargo test --workspace --all-targets --verbose && \
		         cargo build --release && \
		         cargo doc --no-deps"

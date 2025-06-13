build:
	docker build -t ehatrom-ci ./ehatrom

ehatrom_ci:
	docker run --rm -it \
		-v "$(PWD)/ehatrom":/ehatrom \
		-w /ehatrom \
		ehatrom-ci \
		bash -c "cargo +nightly clippy --workspace --all-targets -- -D warnings && \
		         cargo build --workspace --all-targets --verbose && \
		         cargo test --workspace --all-targets --verbose"
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

ehatrom_ci_local:
	cd ehatrom && \
	cargo clippy --workspace --all-targets -- -D warnings && \
	cargo build --workspace --all-targets --verbose && \
	cargo test --workspace --all-targets --verbose

ocultum_ci_local:
	# Установка nightly версии Rust
	rustup install nightly; \
	rustup default nightly; \
	rustup component add --toolchain nightly rustfmt; \
	rustup component add --toolchain nightly clippy; \
	# Проверка формата
	cargo +nightly fmt -- --check; \
	# Запуск Clippy
	cargo +nightly clippy --workspace --all-targets -- -D warnings; \
	# Сборка проекта
	cargo build --workspace --all-targets --verbose; \
	# Запуск тестов
	cargo test --workspace --all-targets --verbose
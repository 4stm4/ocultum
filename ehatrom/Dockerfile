# Dockerfile for local Rust CI (ehatrom)
FROM rust:1.77 AS builder

WORKDIR /workspace/ehatrom

# Установить grcov и llvm-tools-preview для покрытия
RUN rustup component add llvm-tools-preview && \
    cargo install grcov

# Копируем только Cargo.toml и Cargo.lock для кэширования зависимостей
COPY ehatrom/Cargo.toml ehatrom/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# Копируем исходники
COPY ehatrom .

# Форматирование
RUN cargo fmt -- --check

# Clippy
RUN cargo clippy --workspace --all-targets -- -D warnings

# Сборка и тесты
RUN cargo build --workspace --all-targets --verbose
RUN cargo test --workspace --all-targets --verbose

# Покрытие
RUN cargo test --workspace --all-targets --no-fail-fast -- --nocapture && \
    grcov . -s . -t lcov --llvm --branch --ignore-not-existing -o lcov.info

# Артефакты (опционально)
FROM scratch AS artifact
COPY --from=builder /workspace/ehatrom/lcov.info /lcov.info
COPY --from=builder /workspace/ehatrom/target/release /release

# Этап сборки CI
FROM rust:1.87 AS builder

# Устанавливаем рабочую директорию
WORKDIR /workspace/ehatrom

# Устанавливаем необходимые компоненты для Rust nightly
RUN rustup default nightly && \
    rustup component add rustfmt --toolchain nightly && \
    rustup component add clippy --toolchain nightly

# Копируем только Cargo.toml и Cargo.lock для кэширования зависимостей
COPY ehatrom/Cargo.toml ehatrom/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo fetch

# Копируем исходный код
COPY ehatrom/src ./src/

# Проверка форматирования
RUN cargo +nightly fmt -- --check

# Линтинг с Clippy
RUN cargo +nightly clippy --workspace --all-targets -- -D warnings

# Сборка всех целей
RUN cargo build --workspace --all-targets --verbose

# Запуск тестов
RUN cargo test --workspace --all-targets --verbose

# Этап сборки релизных артефактов
FROM rust:nightly AS release

WORKDIR /workspace/ehatrom

# Копируем зависимости и исходный код из этапа builder
COPY --from=builder /workspace/ehatrom /workspace/ehatrom

# Сборка релизной версии
RUN cargo build --release --workspace --verbose

# Финальный этап для артефактов
FROM scratch AS artifact

# Копируем релизные бинарные файлы
COPY --from=release /workspace/ehatrom/target/release/ehatrom /ehatrom
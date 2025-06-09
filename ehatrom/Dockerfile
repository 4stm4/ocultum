# Базовый образ для сборки
FROM rust:1.87.0 AS builder

WORKDIR /workspace/ehatrom

# Установка зависимостей для grcov
RUN apt-get update && apt-get install -y llvm && rm -rf /var/lib/apt/lists/*
RUN rustup component add llvm-tools-preview && \
    cargo install grcov

# Кэширование зависимостей
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Копирование исходников
COPY src ./src/

# Форматирование
RUN cargo fmt -- --check || true

# Проверка линтером
RUN cargo clippy --workspace --all-targets -- -D warnings || true

# Сборка и тесты
RUN cargo build --workspace --all-targets --verbose
RUN cargo test --workspace --all-targets --verbose || true

# Генерация отчета о покрытии
RUN cargo test --workspace --all-targets --no-fail-fast -- --nocapture && \
    grcov . -s . -t lcov --llvm --branch --ignore-not-existing -o lcov.info || true

# Финальный этап для артефактов
FROM scratch AS artifact
COPY --from=builder /workspace/ehatrom/lcov.info /lcov.info
COPY --from=builder /workspace/ehatrom/target/release /release
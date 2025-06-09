FROM rustlang/rust:nightly

WORKDIR /ehatrom

# Копируем весь текущий каталог (предполагаем, что он содержит Cargo.toml и src/)
COPY . .

# Устанавливаем компоненты rustfmt и clippy
RUN rustup component add rustfmt clippy

# Проверка форматирования
RUN cargo +nightly fmt -- --check

# Анализ кода с clippy
RUN cargo +nightly clippy --workspace --all-targets -- -D warnings

# Сборка
RUN cargo +nightly build --workspace --all-targets --verbose

# Тесты
RUN cargo +nightly test --workspace --all-targets --verbose

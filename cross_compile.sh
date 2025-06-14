#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Install dependencies for cross-compilation
if ! command -v arm-linux-gnueabihf-gcc &> /dev/null
then
    echo "arm-linux-gnueabihf-gcc не установлен. Установите его через пакетный менеджер."
    exit 1
fi

# Variables
TARGET="armv7-unknown-linux-gnueabihf"
# Name of the resulting binary matches the package name
BINARY_NAME="ocultum"

# Build the project
cargo build --release --target $TARGET

# Locate the binary
BINARY_PATH="target/$TARGET/release/$BINARY_NAME"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Ошибка: бинарный файл не найден по пути $BINARY_PATH"
    exit 1
fi

echo "Кросс-компиляция завершена успешно. Бинарный файл находится по пути: $BINARY_PATH"

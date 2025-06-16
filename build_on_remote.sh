#!/bin/bash
# build_on_remote.sh
# Скрипт для удаленной сборки на Raspberry Pi Zero 2W
# Требует настроенного SSH-доступа к устройству

set -e

# Параметры подключения (измените на свои)
REMOTE_HOST="alex@192.168.88.140"
REMOTE_DIR="~/ocultum"
REMOTE_PORT=22

# Проверка SSH соединения
echo "=== Проверка SSH соединения ==="
ssh -p $REMOTE_PORT $REMOTE_HOST "echo SSH соединение работает"

# Создание директории на удаленном устройстве, если она не существует
echo "=== Создание директории проекта на удаленном устройстве ==="
ssh -p $REMOTE_PORT $REMOTE_HOST "mkdir -p $REMOTE_DIR"

# Синхронизация файлов проекта
echo "=== Синхронизация файлов проекта ==="
rsync -avz --exclude "target" --exclude ".git" -e "ssh -p $REMOTE_PORT" ./ $REMOTE_HOST:$REMOTE_DIR/

# Установка необходимых зависимостей (если необходимо)
echo "=== Проверка и установка зависимостей ==="
ssh -p $REMOTE_PORT $REMOTE_HOST "cd $REMOTE_DIR && \
    if ! command -v cargo &> /dev/null; then \
        echo 'Устанавливаем Rust...'; \
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
        source \$HOME/.cargo/env; \
    fi && \
    if ! dpkg -l | grep -q libi2c-dev; then \
        echo 'Устанавливаем libi2c-dev...'; \
        sudo apt-get update && sudo apt-get install -y libi2c-dev; \
    fi"

# Сборка проекта на удаленном устройстве
echo "=== Сборка проекта на Raspberry Pi ==="
ssh -p $REMOTE_PORT $REMOTE_HOST "cd $REMOTE_DIR && \
    source \$HOME/.cargo/env && \
    cargo build --release"

# Запуск (если указан параметр run)
if [ "$1" == "run" ]; then
    echo "=== Запуск проекта на Raspberry Pi ==="
    ssh -p $REMOTE_PORT $REMOTE_HOST "cd $REMOTE_DIR && \
        source \$HOME/.cargo/env && \
        sudo ./target/release/ocultum"
fi

echo "=== Сборка завершена успешно ==="
if [ "$1" != "run" ]; then
    echo "Для запуска используйте команду: ./build_on_remote.sh run"
fi

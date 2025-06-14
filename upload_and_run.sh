#!/bin/bash
set -e

# Обновление репозитория
echo "=== git pull ==="
git pull

# Сборка проекта
echo "=== build ==="
cargo build --release --target=aarch64-unknown-linux-gnu

# Загрузка и запуск на Raspberry Pi Zero 2W
echo "=== upload and run ==="
scp target/aarch64-unknown-linux-gnu/release/ocultum pi@raspberrypi:/home/pi/ocultum
ssh pi@raspberrypi "chmod +x /home/pi/ocultum && /home/pi/ocultum"

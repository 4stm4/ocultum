# 4STM4 ocultum
#!/bin/bash
# update_and_run.sh — автоматизация git pull, сборки и запуска
set -e

echo "=== GIT PULL ==="
git pull

echo "=== CARGO BUILD ==="
cargo build

echo "=== CARGO RUN ==="
cargo run || true

echo "=== EEPROM INFO EXAMPLE ==="
echo "Чтобы вывести содержимое дампа EEPROM, используйте:"
echo "sudo ./target/release/ehatrom show dump.bin"

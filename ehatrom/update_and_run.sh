# 4STM4 ocultum
#!/bin/bash
# update_and_run.sh — автоматизация git pull, сборки и демонстрации функциональности
set -e

echo "=== GIT PULL ==="
git pull

echo "=== CARGO BUILD ==="
cargo build

echo "=== CREATING TEST EEPROM FILES ==="
echo "Создаём тестовые EEPROM файлы с помощью examples..."
cargo run --example create_simple
cargo run --example create_test

echo "=== RUNNING TESTS ==="
echo "Запускаем все тесты для проверки качества..."
cargo test --quiet

echo "=== DEMONSTRATING CLI ==="
echo "Демонстрируем работу CLI с созданными файлами:"

echo ""
echo "1. Показываем содержимое простого EEPROM файла:"
cargo run -- show tests/data/simple.eep || echo "Ошибка: файл не содержит обязательных атомов"

echo ""
echo "2. Показываем содержимое полного EEPROM файла:"
cargo run -- show tests/data/test.eep

echo ""
echo "=== USAGE EXAMPLES ==="
echo "Доступные команды CLI:"
echo "  cargo run -- show <file.eep>          # Показать содержимое EEPROM файла"
echo "  cargo run -- read <dev> <addr> <out>  # Читать EEPROM с I2C устройства (только Linux)"
echo "  cargo run -- write <dev> <addr> <in>  # Записать EEPROM на I2C устройство (только Linux)"
echo ""
echo "Примеры создания EEPROM файлов:"
echo "  cargo run --example create_simple     # Создать простой EEPROM файл"
echo "  cargo run --example create_test       # Создать полный тестовый EEPROM файл"
echo ""
echo "Запуск тестов и проверок качества:"
echo "  cargo test                            # Запустить все тесты"
echo "  cargo clippy                          # Проверить код линтером"
echo "  cargo fmt                             # Отформатировать код"

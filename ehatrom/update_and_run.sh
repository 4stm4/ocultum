# 4STM4 ocultum
#!/bin/bash
# update_and_run.sh — демонстрация ehatrom библиотеки с примерами и реальным EEPROM на Raspberry Pi HAT
set -e

echo "================================================================"
echo "         EHATROM - Raspberry Pi HAT EEPROM Library Demo        "
echo "================================================================"

echo ""
echo "=== GIT PULL ==="
git pull

echo ""
echo "=== CARGO BUILD ==="
cargo build --release --features=linux

echo ""
echo "=== RUNNING TESTS ==="
echo "Запускаем все тесты для проверки качества..."
cargo test --quiet --features=linux
echo "✅ Все тесты пройдены успешно"

echo ""
echo "=== ДЕМОНСТРАЦИЯ БИБЛИОТЕКИ: СОЗДАНИЕ EEPROM ФАЙЛОВ ==="
echo ""
echo "📝 Демонстрируем создание различных типов EEPROM файлов..."

echo ""
echo "1️⃣  Создание простого минимального EEPROM (create_simple.rs):"
echo "    - Только заголовок и CRC"
echo "    - Минимальный размер (16 байт)"
echo "    - Подходит для базовой HAT идентификации"
echo ""
cargo run --example create_simple
echo ""

echo "2️⃣  Создание полного тестового EEPROM (create_test.rs):"
echo "    - Заголовок + VendorInfo + GPIO Map"
echo "    - Содержит метаданные производителя"
echo "    - Настройки GPIO для HAT"
echo ""
cargo run --example create_test
echo ""

echo "3️⃣  Создание продвинутого EEPROM с Device Tree (create_advanced.rs):"
echo "    - Полная структура с DT blob"
echo "    - Настройки GPIO с конкретными пинами"
echo "    - Device Tree overlay для HAT"
echo ""
cargo run --example create_advanced
echo ""

echo "4️⃣  Создание EEPROM с кастомными атомами (create_custom_atoms.rs):"
echo "    - Демонстрация пользовательских атомов"
echo "    - Конфигурационные данные"
echo "    - Калибровочные параметры"
echo "    - Таблицы поиска и версионная информация"
echo ""
cargo run --example create_custom_atoms
echo ""

echo "=== АНАЛИЗ СОЗДАННЫХ ФАЙЛОВ ==="
echo ""
echo "📊 Анализируем созданные EEPROM файлы:"

echo ""
echo "🔍 Простой EEPROM (tests/data/simple.eep):"
if [ -f "tests/data/simple.eep" ]; then
    ./target/release/ehatrom show tests/data/simple.eep
else
    echo "❌ Файл simple.eep не найден"
fi

echo ""
echo "🔍 Полный тестовый EEPROM (tests/data/test.eep):"
if [ -f "tests/data/test.eep" ]; then
    ./target/release/ehatrom show tests/data/test.eep
else
    echo "❌ Файл test.eep не найден"
fi

echo ""
echo "🔍 3. Продвинутый EEPROM с Device Tree (tests/data/advanced.eep):"
if [ -f "tests/data/advanced.eep" ]; then
    ./target/release/ehatrom show tests/data/advanced.eep
else
    echo "❌ Файл advanced.eep не найден"
fi

echo ""
echo "🔍 4. EEPROM с кастомными атомами (tests/data/custom_atoms.eep):"
if [ -f "tests/data/custom_atoms.eep" ]; then
    ./target/release/ehatrom show tests/data/custom_atoms.eep
else
    echo "❌ Файл custom_atoms.eep не найден"
fi

echo ""
echo "=== DETECTING EEPROM HAT ==="
echo "🔌 Проверяем наличие EEPROM HAT на I2C шине..."

# Проверяем доступность I2C устройств
if command -v i2cdetect >/dev/null 2>&1; then
    echo "Сканируем I2C шину 1 (HAT EEPROM обычно на адресе 0x50):"
    sudo i2cdetect -y 1 | grep -E "(50|UU)" || echo "EEPROM не найден на стандартном адресе 0x50"
else
    echo "Утилита i2cdetect не найдена. Устанавливаем i2c-tools:"
    sudo apt-get update && sudo apt-get install -y i2c-tools
fi

echo ""
echo "=== РАБОТА С РЕАЛЬНЫМ EEPROM HAT ==="
echo "🔧 Работаем с реальным EEPROM на HAT (если подключен)..."

# Стандартные параметры для HAT EEPROM
I2C_DEVICE="/dev/i2c-1"
EEPROM_ADDR="0x50"
BACKUP_FILE="eeprom_backup_$(date +%Y%m%d_%H%M%S).bin"
TEST_FILE="tests/data/test.eep"

echo ""
echo "💾 ШАГИ РАБОТЫ С РЕАЛЬНЫМ EEPROM:"
echo "   1. Создание резервной копии"
echo "   2. Анализ существующего содержимого"
echo "   3. Запись тестового EEPROM"
echo "   4. Проверка записанных данных"
echo "   5. Восстановление оригинального EEPROM"

echo ""
echo "1️⃣  Создаём резервную копию существующего EEPROM:"
if sudo ./target/release/ehatrom read "$I2C_DEVICE" "$EEPROM_ADDR" "$BACKUP_FILE" 2>/dev/null; then
    echo "✅ EEPROM прочитан и сохранён в $BACKUP_FILE"
    
    echo ""
    echo "2️⃣  Показываем содержимое существующего EEPROM:"
    echo "📋 Анализ оригинального EEPROM с HAT:"
    ./target/release/ehatrom show "$BACKUP_FILE" || echo "⚠️  EEPROM содержит некорректные данные"
    
    echo ""
    echo "3️⃣  Записываем тестовый EEPROM на устройство:"
    echo "🔄 Демонстрация записи нашего тестового EEPROM..."
    if sudo ./target/release/ehatrom write "$I2C_DEVICE" "$EEPROM_ADDR" "$TEST_FILE"; then
        echo "✅ Тестовый EEPROM записан"
        
        echo ""
        echo "4️⃣  Читаем и проверяем записанные данные:"
        VERIFY_FILE="eeprom_verify.bin"
        if sudo ./target/release/ehatrom read "$I2C_DEVICE" "$EEPROM_ADDR" "$VERIFY_FILE"; then
            echo "✅ Данные прочитаны для проверки"
            
            echo ""
            echo "📊 Показываем содержимое записанного EEPROM:"
            ./target/release/ehatrom show "$VERIFY_FILE"
            
            echo ""
            echo "5️⃣  Восстанавливаем оригинальный EEPROM:"
            echo "🔧 Возвращаем оригинальное содержимое EEPROM..."
            if sudo ./target/release/ehatrom write "$I2C_DEVICE" "$EEPROM_ADDR" "$BACKUP_FILE"; then
                echo "✅ Оригинальный EEPROM восстановлен"
                rm -f "$VERIFY_FILE"
            else
                echo "❌ ОШИБКА: Не удалось восстановить оригинальный EEPROM!"
                echo "   Резервная копия сохранена в: $BACKUP_FILE"
            fi
        else
            echo "❌ Ошибка чтения для проверки"
        fi
    else
        echo "❌ Ошибка записи тестового EEPROM"
    fi
else
    echo "❌ Не удалось прочитать EEPROM. Возможные причины:"
    echo "   - HAT не подключен"
    echo "   - I2C не включен (sudo raspi-config -> Interface Options -> I2C)"
    echo "   - EEPROM не на стандартном адресе 0x50"
    echo "   - Нет прав доступа (запустите с sudo)"
    
    echo ""
    echo "📁 Запускаем демонстрацию с локальными файлами:"
    echo ""
    echo "📊 Показываем содержимое тестового EEPROM файла:"
    ./target/release/ehatrom show "$TEST_FILE"
fi

echo ""
echo "================================================================"
echo "                       СПРАВОЧНАЯ ИНФОРМАЦИЯ                    "
echo "================================================================"

echo ""
echo "🚀 ВОЗМОЖНОСТИ БИБЛИОТЕКИ EHATROM:"
echo ""
echo "  📦 Основные функции:"
echo "    • Чтение/запись EEPROM с реальных HAT через I2C"
echo "    • Парсинг и создание HAT EEPROM структур"
echo "    • Валидация CRC32 (IEEE 802.3)"
echo "    • Поддержка всех стандартных HAT атомов"
echo "    • Zero external dependencies (bare-metal готово)"
echo ""
echo "  🔧 CLI команды:"
echo "    • read   - Чтение EEPROM с I2C устройства"
echo "    • write  - Запись EEPROM на I2C устройство"
echo "    • show   - Анализ и отображение EEPROM файла"
echo ""
echo "  📝 Примеры создания EEPROM:"
echo "    • create_simple.rs      - Минимальный EEPROM (16 байт)"
echo "    • create_test.rs        - Полный EEPROM с метаданными"
echo "    • create_advanced.rs    - EEPROM с Device Tree blob"
echo "    • create_custom_atoms.rs- EEPROM с пользовательскими атомами"
echo ""

echo "📚 ПРИМЕРЫ ИСПОЛЬЗОВАНИЯ:"
echo ""
echo "  📖 Работа с реальным EEPROM:"
echo "    sudo ./target/release/ehatrom read /dev/i2c-1 0x50 backup.bin"
echo "    ./target/release/ehatrom show backup.bin"
echo "    sudo ./target/release/ehatrom write /dev/i2c-1 0x50 new_eeprom.bin"
echo ""
echo "  🏗️  Создание EEPROM файлов:"
echo "    cargo run --example create_simple       # Минимальный EEPROM"
echo "    cargo run --example create_test         # Полный тестовый EEPROM"
echo "    cargo run --example create_advanced     # EEPROM с Device Tree"
echo "    cargo run --example create_custom_atoms # EEPROM с кастомными атомами"
echo ""
echo "  🔍 Диагностика I2C:"
echo "    sudo i2cdetect -y 1                   # Сканировать I2C шину 1"
echo "    lsmod | grep i2c                      # Проверить модули I2C"
echo "    sudo raspi-config                     # Включить I2C интерфейс"
echo ""

echo "⚠️  ВАЖНЫЕ ПРЕДУПРЕЖДЕНИЯ:"
echo ""
echo "  🛡️  БЕЗОПАСНОСТЬ:"
echo "    • ВСЕГДА делайте резервную копию перед записью"
echo "    • EEPROM содержит критическую информацию о HAT"
echo "    • Неправильная запись может повредить HAT"
echo "    • Используйте sudo только для I2C операций"
echo ""
echo "  🔧 ТЕХНИЧЕСКИЕ ТРЕБОВАНИЯ:"
echo "    • Raspberry Pi с включенным I2C"
echo "    • Права доступа к /dev/i2c-1"
echo "    • HAT с EEPROM на стандартном адресе 0x50"
echo ""

echo "🎯 СТРУКТУРА ПРОЕКТА:"
echo ""
echo "  📁 Файлы и папки:"
echo "    • src/lib.rs           - Основная библиотека"
echo "    • src/main.rs          - CLI интерфейс"
echo "    • src/utils/crc32.rs   - Кастомная CRC32 реализация"
echo "    • examples/            - Примеры создания EEPROM"
echo "    • tests/               - Тесты (16 тестов)"
echo "    • tests/data/          - Тестовые EEPROM файлы"
echo ""

echo ""
echo "================================================================"
echo "                  ДЕМОНСТРАЦИЯ ЗАВЕРШЕНА!                      "
echo "================================================================"
echo ""
echo "✅ Библиотека ehatrom готова к использованию!"
echo "📖 Подробная документация в README.md"
echo "🚀 Для реального использования подключите Raspberry Pi HAT"
echo ""

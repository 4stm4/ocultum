# Архитектура Ocultum

Проект реализует порт-адаптерную архитектуру (Hexagonal Architecture):

## Структура

```
src/
├── core/           # Ядро (бизнес-логика)
│   ├── domain.rs   # Доменные модели
│   └── service.rs  # Основной сервис
├── ports/          # Порты (интерфейсы)
│   └── mod.rs      # EepromReader, DisplayWriter, DisplayDetector
├── adapters/       # Адаптеры (реализации)
│   ├── ehatrom_adapter.rs    # Для работы с EEPROM
│   └── ssd1306_adapter.rs    # Для работы с дисплеем
└── main.rs         # Точка входа и dependency injection
```

## Принципы

1. **Ядро независимо** от инфраструктуры
2. **Порты** определяют контракты взаимодействия
3. **Адаптеры** реализуют порты для конкретных технологий
4. **Dependency Injection** в main.rs

## Преимущества

- Легко тестировать (mock-адаптеры)
- Легко заменять технологии
- Чистая архитектура
- Разделение ответственности

## Использование

```rust
// Создаем адаптеры
let eeprom_adapter = EhatromAdapter::new();
let display_adapter = Ssd1306Adapter::new();

// Создаем сервис
let service = OcultumService::new(eeprom_adapter, display_adapter, display_adapter);

// Запускаем
service.run()?;
```

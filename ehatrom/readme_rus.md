# ehatrom — библиотека для работы с HAT EEPROM Raspberry Pi

`ehatrom` — это библиотека на Rust для чтения, записи и генерации содержимого EEPROM для Raspberry Pi HAT (Hardware Attached on Top) через I2C. Поддерживает корректную сериализацию/десериализацию структуры, работу с атомами (VendorInfo, GPIO Map, DTBlob, пользовательские), чтение/запись с 2-байтовым offset и page write, а также удобный вывод содержимого.

## Возможности
- Чтение и запись EEPROM HAT через I2C (с поддержкой page write и 2-байтового offset)
- Сериализация и парсинг структуры EEPROM согласно официальной спецификации Raspberry Pi HAT
- Удобный вывод содержимого, включая строковые поля
- CLI-пример для чтения/записи/вывода EEPROM
- Поддержка пользовательских атомов и контроль целостности CRC32

## Структуры
- `EepromHeader` — заголовок EEPROM
- `AtomHeader` — заголовок атома
- `VendorInfoAtom` — информация о производителе и продукте
- `GpioMapAtom` — карта GPIO (28 пинов на банк)
- `DtBlobAtom` — blob с device tree
- `Eeprom` — вся структура EEPROM

### Почему 28 пинов в GpioMapAtom?
28 пинов соответствуют GPIO0–GPIO27 стандартного 40-пинового разъёма Raspberry Pi. Это ровно столько, сколько пользовательских GPIO доступно на обычных моделях. Для расширенных плат (Compute Module) может быть добавлен второй атом (GpioMapBank1).

## Пример использования

```rust
use ehatrom::{Eeprom, VendorInfoAtom, GpioMapAtom};

// Создание структуры VendorInfoAtom
let vendor_info = VendorInfoAtom::new(
    0x1234, // vendor_id
    0x5678, // product_id
    1,      // product_ver
    "MyVendor", // vendor (строка любой длины)
    "MyHAT",    // product (строка любой длины)
    [0u8; 16],   // uuid
);

// Заполнение карты GPIO: все не используются (0), GPIO4 — вход (0x01), GPIO17 — выход (0x02)
let mut pins = [0u8; 28];
pins[4] = 0x01;   // GPIO4 — input
pins[17] = 0x02;  // GPIO17 — output
let gpio_map = GpioMapAtom { flags: 0, pins };

let mut eeprom = Eeprom {
    header: Default::default(),
    vendor_info,
    gpio_map_bank0: gpio_map,
    dt_blob: None,
    gpio_map_bank1: None,
    custom_atoms: Vec::new(),
};
eeprom.update_header();

// Сериализация в байты
let bytes = eeprom.serialize();

// Сериализация с CRC32
let bytes_with_crc = eeprom.serialize_with_crc();

// Запись в EEPROM через I2C
// ehatrom::write_to_eeprom_i2c(&bytes_with_crc, "/dev/i2c-1", 0x50)?;

// Чтение из EEPROM с проверкой CRC
// let mut buf = vec![0u8; 256];
// ehatrom::read_from_eeprom_i2c(&mut buf, "/dev/i2c-1", 0x50, 0)?;
// if Eeprom::verify_crc(&buf) {
//     let eeprom = Eeprom::from_bytes(&buf[..buf.len()-4])?;
//     println!("{:?}", eeprom);
// } else {
//     println!("CRC check failed!");
// }

// Добавление пользовательского атома (например, с настройками или серийным номером)
let custom_data = b"serial:1234567890".to_vec();
eeprom.add_custom_atom(0x80, custom_data);

// Добавление пользовательского атома с настройками (например, API-адреса)
let api_url = b"api_url:https://api.example.com/v1".to_vec();
eeprom.add_custom_atom(0x80, api_url);
let api_key = b"api_key:SECRET123456".to_vec();
eeprom.add_custom_atom(0x81, api_key);
```

## Формат поля pins
Каждый байт массива `pins` определяет назначение соответствующего GPIO:
- 0x00 — не используется
- 0x01 — вход
- 0x02 — выход
- ... (см. спецификацию HAT EEPROM)

## Ссылки
- [Официальная спецификация HAT EEPROM](https://github.com/raspberrypi/hats/blob/master/eeprom-format.md)

## Лицензия
MIT

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

// Условная компиляция для Linux
#[cfg(feature = "linux")]
use linux_embedded_hal::{I2cdev, Delay};

// Заглушка для I2C и Delay для не-Linux систем (например, bare metal)
#[cfg(not(feature = "linux"))]
struct MockI2C;
#[cfg(not(feature = "linux"))]
impl embedded_hal::i2c::ErrorType for MockI2C {
    type Error = core::convert::Infallible;
}
#[cfg(not(feature = "linux"))]
impl embedded_hal::i2c::I2c for MockI2C {
    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(()) // В заглушке просто возвращаем Ok
    }
}

#[cfg(not(feature = "linux"))]
struct MockDelay;
#[cfg(not(feature = "linux"))]
impl embedded_hal::delay::DelayNs for MockDelay {
    fn delay_ns(&mut self, _ns: u32) {}
    fn delay_ms(&mut self, _ms: u32) {}
    fn delay_us(&mut self, _us: u32) {}
}


#[cfg(target_os = "none")] // Для bare-metal
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let i2c = MockI2C; // Используем заглушку для bare-metal примера
    let delay = MockDelay;
    init_oled(i2c, delay);

    loop {
        // Здесь может быть ваш цикл для bare-metal
    }
}

#[cfg(not(target_os = "none"))] // Для систем с ОС (например, Linux)
fn main() {
    println!("Host mode initialized");

    #[cfg(feature = "linux")]
    {
        println!("Linux feature enabled, attempting to use real I2C.");
        // Укажите правильный путь к вашему I2C устройству и адрес дисплея
        // Обычно это /dev/i2c-0, /dev/i2c-1 и т.д.
        // Адрес дисплея SSD1306 обычно 0x3C или 0x3D
        match I2cdev::new("/dev/i2c-1") {
            Ok(i2c) => {
                let delay = Delay; // Используем Delay из linux-embedded-hal
                init_oled(i2c, delay);
            }
            Err(e) => {
                eprintln!("Failed to open I2C device /dev/i2c-1: {:?}", e);
                eprintln!("Please check the I2C device path and permissions.");
            }
        }
    }

    #[cfg(not(feature = "linux"))]
    {
        println!("Linux feature NOT enabled, using MockI2C.");
        let i2c = MockI2C;
        let delay = MockDelay;
        init_oled(i2c, delay);
    }
}

// Теперь init_oled принимает и I2C, и Delay
fn init_oled<I2C, D>(i2c: I2C, mut delay: D)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug, // Добавляем требование Debug для ошибки I2C
    D: embedded_hal::delay::DelayNs,
{
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if disp.init().is_err() {
        eprintln!("Failed to initialize OLED display.");
        return;
    }
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("Failed to clear OLED display.");
        return;
    }

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    if Text::with_baseline("Hello, Rust!", Point::new(0, 10), text_style, Baseline::Top) // Сместил текст чуть ниже для лучшей видимости
        .draw(&mut disp)
        .is_err()
    {
        eprintln!("Failed to draw text on OLED.");
        return;
    }

    if disp.flush().is_err() {
        eprintln!("Failed to flush OLED display buffer.");
        return;
    }

    println!("OLED display initialized and text should be visible.");

    // Небольшая задержка, чтобы успеть увидеть текст перед завершением программы (если это не _start)
    #[cfg(not(target_os = "none"))]
    delay.delay_ms(5000);
}

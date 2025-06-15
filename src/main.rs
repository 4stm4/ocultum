#![cfg_attr(all(not(feature = "linux"), target_os = "none"), no_std)]
#![cfg_attr(all(not(feature = "linux"), target_os = "none"), no_main)]

#[cfg(all(not(feature = "linux"), target_os = "none", not(test)))]
// Используется только в no_std сборках не для тестов для bare-metal
use core::panic::PanicInfo;

// Условная компиляция для Linux
#[cfg(feature = "linux")]
use linux_embedded_hal::{Delay, I2cdev};

#[cfg(feature = "linux")] // Оборачиваем импорты, используемые в init_oled
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10}, // Порядок изменен согласно fmt
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
#[cfg(feature = "linux")] // Оборачиваем импорты, используемые в init_oled
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

// Заглушка для I2C и Delay для не-Linux систем (например, bare metal)
#[cfg(not(feature = "linux"))]
#[allow(dead_code)]
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
#[allow(dead_code)]
struct MockDelay;
#[cfg(not(feature = "linux"))]
impl embedded_hal::delay::DelayNs for MockDelay {
    fn delay_ns(&mut self, _ns: u32) {}
    fn delay_ms(&mut self, _ms: u32) {}
    fn delay_us(&mut self, _us: u32) {}
}

#[cfg(all(not(feature = "linux"), target_os = "none"))] // Для bare-metal и когда фича linux не активна
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Тут может быть инициализация для MockI2C/MockDelay, если они нужны в _start
    loop {}
}

#[cfg(feature = "linux")] // Для систем с ОС (например, Linux) и когда фича linux активна
fn main() {
    eprintln!("P1: Host mode initialized (stderr)");

    eprintln!("P2: Linux feature IS enabled. Attempting real I2C. (stderr)"); // Теперь это утверждение всегда верно, если main компилируется

    // Непосредственно выполняем код для Linux, так как main компилируется только с фичей "linux"
    match I2cdev::new("/dev/i2c-1") {
        Ok(i2c) => {
            eprintln!("P3.1: I2C device /dev/i2c-1 opened successfully. (stderr)");
            let delay = Delay;
            init_oled(i2c, delay);
        }
        Err(e) => {
            eprintln!(
                "P3.2: ERROR - Failed to open I2C device /dev/i2c-1: {:?} (stderr)",
                e
            );
            eprintln!("P3.3: Please check the I2C device path and permissions. (stderr)");
        }
    }

    eprintln!("P5: After I2C attempt. Main is ending. (stderr)"); // Сообщение P4 удалено как недостижимое
}

#[cfg(feature = "linux")] // Оборачиваем всю функцию
fn init_oled<I2C, D>(i2c: I2C, mut delay: D)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug,
    D: embedded_hal::delay::DelayNs,
{
    eprintln!("INIT_OLED: Attempting to initialize display... (stderr)");
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if disp.init().is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to initialize OLED display. (stderr)");
        return;
    }
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to clear OLED display. (stderr)");
        return;
    }

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    if Text::with_baseline("Hello, Rust!", Point::new(0, 10), text_style, Baseline::Top)
        .draw(&mut disp)
        .is_err()
    {
        eprintln!("INIT_OLED: ERROR - Failed to draw text on OLED. (stderr)");
        return;
    }

    if disp.flush().is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to flush OLED display buffer. (stderr)");
        return;
    }

    eprintln!("INIT_OLED: OLED display initialized and text should be visible. (stderr)");

    #[cfg(not(target_os = "none"))]
    delay.delay_ms(5000);
}

#[cfg(all(not(feature = "linux"), not(target_os = "none")))]
fn main() {
    // Эта main будет скомпилирована только для хост-систем (не bare-metal)
    // когда фича "linux" НЕ активна.
    // Она нужна, чтобы cargo build/test на хосте не падали с ошибкой E0601.
    println!("This is a placeholder main for host builds without the 'linux' feature.");
    println!("To run the OLED application, please build with '--features linux'.");
}

#[cfg(all(not(feature = "linux"), target_os = "none", not(test)))]
// Компилируем только для no_std сборок, не являющихся тестами для bare-metal
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

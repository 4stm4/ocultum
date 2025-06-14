use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

// Добавляю недостающие импорты
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder};
use embedded_graphics::text::{Baseline, Text};

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mock_i2c = MockI2C;
    init_oled(mock_i2c);

    loop {
        embedded_hal::delay::DelayMs::delay_ms(&mut (), 1000u32);
    }
}

#[cfg(not(target_os = "none"))]
fn main() {
    println!("Linux mode initialized");
    let mock_i2c = MockI2C;
    init_oled(mock_i2c);
}

fn init_oled<I2C>(i2c: I2C)
where
    I2C: embedded_hal::i2c::I2c,
{
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();
    disp.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello, Rust!", Point::new(0, 0), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();

    disp.flush().unwrap();
}

// Заглушка для объекта I2C
struct MockI2C;
impl embedded_hal::i2c::ErrorType for MockI2C {
    type Error = core::convert::Infallible;
}

impl embedded_hal::i2c::I2c for MockI2C {
    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

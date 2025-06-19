//! Адаптер для работы с SSD1306 дисплеем

use crate::core::domain::{DeviceInfo, DisplayDevice};
use crate::ports::{DisplayDetector, DisplayWriter};

#[cfg(target_os = "linux")]
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

#[cfg(target_os = "linux")]
use embedded_hal::i2c::I2c;

#[cfg(target_os = "linux")]
use linux_embedded_hal::I2cdev;

#[cfg(target_os = "linux")]
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[derive(Debug)]
pub struct SsdError(pub String);

impl std::fmt::Display for SsdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSD1306 Error: {}", self.0)
    }
}

impl std::error::Error for SsdError {}

pub struct Ssd1306Adapter;

impl Ssd1306Adapter {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "linux")]
    fn detect_display_internal(&self, max_bus: u8) -> Option<(u8, u8)> {
        (0..=max_bus).find_map(|bus| {
            let i2c_path = format!("/dev/i2c-{bus}");
            let Ok(mut i2c) = I2cdev::new(&i2c_path) else {
                return None;
            };
            [0x3C, 0x3D]
                .iter()
                .find(|&&addr| {
                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    i2c.transaction(addr, &mut ops).is_ok()
                })
                .map(|&addr| (bus, addr))
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn detect_display_internal(&self, _max_bus: u8) -> Option<(u8, u8)> {
        // Mock для не-Linux платформ
        Some((1, 0x3C))
    }
}

impl DisplayDetector for Ssd1306Adapter {
    type Error = SsdError;

    fn detect_display(&self) -> Result<DisplayDevice, Self::Error> {
        self.detect_display_internal(20)
            .map(|(bus, addr)| DisplayDevice::new(bus, addr))
            .ok_or_else(|| SsdError("Display not found".to_string()))
    }
}

impl DisplayWriter for Ssd1306Adapter {
    type Error = SsdError;

    #[cfg(target_os = "linux")]
    fn display_info(&self, device: &DisplayDevice, info: &DeviceInfo) -> Result<(), Self::Error> {
        eprintln!(
            "Displaying on I2C bus {} address 0x{:02X}",
            device.bus, device.address
        );

        let i2c_path = format!("/dev/i2c-{}", device.bus);
        let i2c =
            I2cdev::new(&i2c_path).map_err(|e| SsdError(format!("Failed to open I2C: {e:?}")))?;

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        display
            .init()
            .map_err(|e| SsdError(format!("Display init failed: {e:?}")))?;
        display
            .clear(BinaryColor::Off)
            .map_err(|e| SsdError(format!("Clear failed: {e:?}")))?;

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            &format!("Vendor: {}", info.vendor),
            Point::new(0, 8),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .map_err(|e| SsdError(format!("Text draw failed: {e:?}")))?;

        Text::with_baseline(
            &format!("Product: {}", info.product),
            Point::new(0, 20),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .map_err(|e| SsdError(format!("Text draw failed: {e:?}")))?;

        display
            .flush()
            .map_err(|e| SsdError(format!("Flush failed: {e:?}")))?;

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn display_info(&self, device: &DisplayDevice, info: &DeviceInfo) -> Result<(), Self::Error> {
        // Mock для не-Linux платформ - просто выводим в консоль
        eprintln!(
            "Mock Display on device bus={}, address=0x{:02X}:",
            device.bus, device.address
        );
        eprintln!("  Vendor: {}", info.vendor);
        eprintln!("  Product: {}", info.product);
        Ok(())
    }
}

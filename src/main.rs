mod adapters;
mod core;
mod ports;

use crate::adapters::{ehatrom_adapter::EhatromAdapter, ssd1306_adapter::Ssd1306Adapter};
use crate::core::service::OcultumService;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Ocultum: Initialization...");

    // Создаем адаптеры
    let eeprom_adapter = EhatromAdapter::new();
    let display_adapter = Ssd1306Adapter::new();
    let detector_adapter = Ssd1306Adapter::new();

    // Создаем сервис с dependency injection
    let service = OcultumService::new(eeprom_adapter, display_adapter, detector_adapter);

    // Запускаем основную логику
    service.run()?;

    eprintln!("Ocultum: Completed successfully!");
    Ok(())
}

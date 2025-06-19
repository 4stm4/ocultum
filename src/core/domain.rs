//! Доменные модели для ocultum

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor: String,
    pub product: String,
}

#[derive(Debug, Clone)]
pub struct DisplayDevice {
    pub bus: u8,
    pub address: u8,
}

impl DeviceInfo {
    pub fn new(vendor: String, product: String) -> Self {
        Self { vendor, product }
    }

    pub fn is_valid(&self) -> bool {
        !self.vendor.is_empty() && !self.product.is_empty()
    }
}

impl DisplayDevice {
    pub fn new(bus: u8, address: u8) -> Self {
        Self { bus, address }
    }
}

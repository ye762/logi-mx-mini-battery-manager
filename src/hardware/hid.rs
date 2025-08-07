use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{debug, warn};

pub struct HIDCommunicator {
    api: HidApi,
}

impl HIDCommunicator {
    pub fn new() -> Result<Self> {
        let api = HidApi::new()
            .context("Failed to initialize HID API")?;
        
        Ok(Self { api })
    }
    
    pub fn get_battery_level(&self, vendor_id: u16, product_id: u16) -> Result<Option<u8>> {
        let device_info = self.api
            .device_list()
            .find(|dev| dev.vendor_id() == vendor_id && dev.product_id() == product_id);
        
        if let Some(info) = device_info {
            let device = info.open_device(&self.api)
                .context("Failed to open HID device")?;
            
            self.read_battery_from_device(&device)
        } else {
            debug!("HID device not found: {:04x}:{:04x}", vendor_id, product_id);
            Ok(None)
        }
    }
    
    fn read_battery_from_device(&self, device: &HidDevice) -> Result<Option<u8>> {
        // Logitech HID++ protocol for battery status
        // This is a simplified implementation - actual protocol may vary
        let mut buf = [0u8; 20];
        
        // HID++ short message: device_index=0xFF, feature_index=0x00, function=0x00
        // This requests basic device information
        buf[0] = 0x10; // Report ID for HID++
        buf[1] = 0xFF; // Device index (unifying receiver)
        buf[2] = 0x00; // Feature index
        buf[3] = 0x00; // Function
        
        match device.write(&buf[..4]) {
            Ok(_) => {
                // Read response
                match device.read_timeout(&mut buf, 1000) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        // Parse battery level from response
                        // This is device-specific and may need adjustment
                        if buf[0] == 0x10 && bytes_read >= 7 {
                            let battery_level = buf[6]; // Battery percentage
                            debug!("Battery level read: {}%", battery_level);
                            Ok(Some(battery_level))
                        } else {
                            warn!("Unexpected HID response format");
                            Ok(None)
                        }
                    }
                    Ok(_) => {
                        warn!("Empty HID response");
                        Ok(None)
                    }
                    Err(e) => {
                        warn!("Failed to read from HID device: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to write to HID device: {}", e);
                Ok(None)
            }
        }
    }
}


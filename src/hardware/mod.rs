pub mod usb;
pub mod hid;
pub mod power;

pub use usb::USBDeviceManager;
pub use hid::HIDCommunicator;
pub use power::PowerController;

// src/hardware/usb.rs
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct USBDevice {
    pub bus: u8,
    pub device: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub sys_path: String,
}

pub struct USBDeviceManager;

impl USBDeviceManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn find_device(&self, vendor_id: u16, product_id: u16) -> Result<Option<USBDevice>> {
        let usb_devices_path = "/sys/bus/usb/devices";
        let entries = fs::read_dir(usb_devices_path)
            .context("Failed to read USB devices directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(device) = self.parse_usb_device(&path, vendor_id, product_id)? {
                return Ok(Some(device));
            }
        }
        
        Ok(None)
    }
    
    fn parse_usb_device(&self, path: &Path, target_vendor: u16, target_product: u16) -> Result<Option<USBDevice>> {
        // Skip root hubs and other non-device entries
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if !dir_name.contains(':') {
            return Ok(None);
        }
        
        let vendor_path = path.join("idVendor");
        let product_path = path.join("idProduct");
        
        if !vendor_path.exists() || !product_path.exists() {
            return Ok(None);
        }
        
        let vendor_str = fs::read_to_string(&vendor_path)
            .context("Failed to read vendor ID")?;
        let product_str = fs::read_to_string(&product_path)
            .context("Failed to read product ID")?;
        
        let vendor_id = u16::from_str_radix(vendor_str.trim(), 16)
            .context("Failed to parse vendor ID")?;
        let product_id = u16::from_str_radix(product_str.trim(), 16)
            .context("Failed to parse product ID")?;
        
        if vendor_id == target_vendor && product_id == target_product {
            // Parse bus and device numbers from directory name
            let parts: Vec<&str> = dir_name.split(['-', ':']).collect();
            if parts.len() >= 2 {
                let bus = parts[0].parse().unwrap_or(0);
                let device_num = self.get_device_number(path)?;
                
                return Ok(Some(USBDevice {
                    bus,
                    device: device_num,
                    vendor_id,
                    product_id,
                    sys_path: path.to_string_lossy().to_string(),
                }));
            }
        }
        
        Ok(None)
    }
    
    fn get_device_number(&self, path: &Path) -> Result<u8> {
        let devnum_path = path.join("devnum");
        if devnum_path.exists() {
            let devnum_str = fs::read_to_string(&devnum_path)
                .context("Failed to read device number")?;
            devnum_str.trim().parse()
                .context("Failed to parse device number")
        } else {
            Ok(0)
        }
    }
}


use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use log::debug;

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

            if let Some(device) = self.check_device_by_uevent(&path, vendor_id, product_id)? {
                return Ok(Some(device));
            }
        }

        Ok(None)
    }

    fn check_device_by_uevent(&self, path: &Path, target_vendor: u16, target_product: u16) -> Result<Option<USBDevice>> {
        let uevent_path = path.join("uevent");

        // Check if uevent file exists
        if !uevent_path.exists() {
            return Ok(None);
        }

        // Read uevent file content
        let uevent_content = fs::read_to_string(&uevent_path)
            .context("Failed to read uevent file")?;

        // Look for PRODUCT line in format: PRODUCT=vendor_id/product_id/version
        for line in uevent_content.lines() {
            if line.starts_with("PRODUCT=") {
                let product_line = &line[8..]; // Remove "PRODUCT=" prefix
                let parts: Vec<&str> = product_line.split('/').collect();

                if parts.len() >= 2 {
                    // Parse vendor_id and product_id from hex strings
                    if let (Ok(vendor_id), Ok(product_id)) = (
                        u16::from_str_radix(parts[0], 16),
                        u16::from_str_radix(parts[1], 16),
                    ) {
                        debug!("Found device: vendor=0x{:04x}, product=0x{:04x} at {:?}",
                               vendor_id, product_id, path);

                        if vendor_id == target_vendor && product_id == target_product {
                            // Extract bus and device numbers from path
                            return self.create_usb_device(path, vendor_id, product_id);
                        }
                    }
                }
                break; // Only process first PRODUCT= line
            }
        }

        Ok(None)
    }

    fn create_usb_device(&self, path: &Path, vendor_id: u16, product_id: u16) -> Result<Option<USBDevice>> {
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Parse bus number from directory name (format: usb1, 1-1, 1-1.2, etc.)
        let bus = if dir_name.starts_with("usb") {
            dir_name[3..].parse().unwrap_or(0)
        } else {
            dir_name.chars().next().unwrap_or('0').to_digit(10).unwrap_or(0) as u8
        };

        let device_num = self.get_device_number(path)?;

        Ok(Some(USBDevice {
            bus,
            device: device_num,
            vendor_id,
            product_id,
            sys_path: path.to_string_lossy().to_string(),
        }))
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
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub thresholds: ThresholdConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub high_threshold: u8,
    pub low_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub use_journal: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: DeviceConfig {
                vendor_id: 0x05e3, // Logitech
                product_id: 0x0608, // MX Mini
                name: "Logitech MX Mini".to_string(),
            },
            thresholds: ThresholdConfig {
                high_threshold: 80,
                low_threshold: 20,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                use_journal: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "/etc/mx-mini-battery-manager/config.json";
        
        match fs::read_to_string(config_path) {
            Ok(content) => {
                serde_json::from_str(&content)
                    .context("Failed to parse configuration file")
            }
            Err(_) => {
                log::info!("Config file not found, using defaults");
                Ok(Self::default())
            }
        }
    }
}


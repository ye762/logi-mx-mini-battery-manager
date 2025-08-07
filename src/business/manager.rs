mod hardware {}

use anyhow::{Context, Result};
use log::{info, warn, error};

use crate::config::Config;
use crate::hardware::{USBDeviceManager, HIDCommunicator, PowerController};

pub struct BatteryManager {
    config: Config,
    usb_manager: USBDeviceManager,
    hid_communicator: HIDCommunicator,
    power_controller: PowerController,
}

#[derive(Debug)]
pub enum Action {
    ChargingEnabled,
    ChargingDisabled,
    NoChange,
    Error,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::ChargingEnabled => write!(f, "charging_enabled"),
            Action::ChargingDisabled => write!(f, "charging_disabled"),
            Action::NoChange => write!(f, "no_change"),
            Action::Error => write!(f, "error"),
        }
    }
}

impl BatteryManager {
    pub fn new(config: Config) -> Result<Self> {
        let hid_communicator = HIDCommunicator::new()
            .context("Failed to initialize HID communicator")?;

        Ok(Self {
            config,
            usb_manager: USBDeviceManager::new(),
            hid_communicator,
            power_controller: PowerController::new(),
        })
    }

    pub async fn check_and_manage(&mut self) -> Result<()> {
        let device_config = &self.config.device;

        match self.usb_manager.find_device(device_config.vendor_id, device_config.product_id)? {
            Some(usb_device) => {
                info!("Device found: {} at {}", device_config.name, usb_device.sys_path);
                self.manage_device_battery(&usb_device).await?;
            }
            None => {
                info!("Device not found via USB: vendor_id=0x{:04x}, product_id=0x{:04x}", 
                     device_config.vendor_id, device_config.product_id);
            }
        }

        Ok(())
    }

    async fn manage_device_battery(&mut self, device: &crate::hardware::usb::USBDevice) -> Result<()> {
        let battery_level = self.hid_communicator
            .get_battery_level(device.vendor_id, device.product_id)?;

        let action = match battery_level {
            Some(level) => {
                let is_charging_enabled = self.power_controller
                    .is_charging_enabled(&device.sys_path)?;

                let should_charge = level < self.config.thresholds.high_threshold;

                let action = if should_charge && !is_charging_enabled {
                    match self.power_controller.set_charging_enabled(&device.sys_path, true) {
                        Ok(()) => Action::ChargingEnabled,
                        Err(e) => {
                            error!("Failed to enable charging: {}", e);
                            Action::Error
                        }
                    }
                } else if !should_charge && is_charging_enabled {
                    match self.power_controller.set_charging_enabled(&device.sys_path, false) {
                        Ok(()) => Action::ChargingDisabled,
                        Err(e) => {
                            error!("Failed to disable charging: {}", e);
                            Action::Error
                        }
                    }
                } else {
                    Action::NoChange
                };

                info!("is_connected_via_usb=true, battery_level={}%, action_done={}", 
                     level, action);

                action
            }
            None => {
                warn!("is_connected_via_usb=true, battery_level=unknown, action_done=error");
                Action::Error
            }
        };

        Ok(())
    }
}
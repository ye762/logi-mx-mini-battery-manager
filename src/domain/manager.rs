mod hardware {}

use std::any::Any;
use std::fmt::{write, Arguments, Display};
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
pub enum PowerEvent {
    ChargingEnabling(u8),
    ChargingDisabling(u8),
    NoChange(u8),
    Error(Option<String>)
}


impl std::fmt::Display for PowerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerEvent::ChargingEnabling(v) => write!(f, "charging_enabled, at {}%", v),
            PowerEvent::ChargingDisabling(v) => write!(f, "charging_disabled, at {}%", v),
            PowerEvent::NoChange(v) => write!(f, "no_change, at {}", v),
            PowerEvent::Error(e) => {
                let msg = e.as_ref().unwrap();
                write!(f, "error: {}", e.as_ref().unwrap())
            }
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
                let new_event = self.resolve_next_event(&usb_device).await?;
                info!("is_connected_via_usb=true, event: {}", new_event);
                self.process_event(new_event, &usb_device)
            }
            None => {
                info!("Device not found via USB: vendor_id=0x{:04x}, product_id=0x{:04x}", 
                     device_config.vendor_id, device_config.product_id);
            }
        }

        Ok(())
    }

    fn process_event(&self, event: PowerEvent, device: &crate::hardware::usb::USBDevice) {
        match event {
            PowerEvent::ChargingEnabling(_) => {
                info!("Charging enabling");
                self.power_controller.set_charging_enabled(&device.sys_path).unwrap_or({
                    error!("Failed to enable charging.");
                });
                info!("Charging enabled in device at {}", device.sys_path);
            }
            PowerEvent::ChargingDisabling(v) => {
                info!("Charging disabling in device at {}...", device.sys_path);
                self.power_controller.set_charging_disabled(&device.sys_path).unwrap_or({
                    error!("Failed to disable charging.");
                });
                info!("Charging disabled for device at {}", device.sys_path);
            }
            PowerEvent::NoChange(_) => {
                info!("Do nothing in device at {}", device.sys_path);
            }
            PowerEvent::Error(_) => {
                error!("Error occurred in device at {}", device.sys_path);
            }
        }
    }

    async fn resolve_next_event(&mut self, device: &crate::hardware::usb::USBDevice) -> Result<PowerEvent> {

        let battery_level_optional= self.hid_communicator.get_battery_level(
            device.vendor_id, device.product_id
        )?;

        let nextEvent: PowerEvent = match Some(battery_level_optional) {
            Some(battery_level) => {
                let actual_battery_level: u8 = battery_level.unwrap();
                let event = if actual_battery_level < self.config.thresholds.high_threshold {
                    PowerEvent::ChargingEnabling(actual_battery_level)
                } else {
                    PowerEvent::ChargingDisabling(actual_battery_level)
                };
                event
            }
            None => PowerEvent::Error(None)
        };

        Ok(nextEvent)
    }
}
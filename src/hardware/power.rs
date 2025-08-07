use anyhow::{Context, Result};
use std::fs;
use log::{debug, warn};

pub struct PowerController;

impl PowerController {
    pub fn new() -> Self {
        Self
    }
    
    pub fn set_charging_enabled(&self, sys_path: &str) -> Result<()> {
        let autosuspend_path = format!("{}/power/autosuspend", sys_path);
        let control_path = format!("{}/power/control", sys_path);
        self.enable_charging(&autosuspend_path, &control_path)
    }

    pub fn set_charging_disabled(&self, sys_path: &str) -> Result<()> {
        let autosuspend_path = format!("{}/power/autosuspend", sys_path);
        let control_path = format!("{}/power/control", sys_path);
        self.disable_charging(&autosuspend_path, &control_path)
    }
    
    fn enable_charging(&self, autosuspend_path: &str, control_path: &str) -> Result<()> {
        // Enable autosuspend and set control to auto for normal charging
        if let Err(e) = fs::write(autosuspend_path, "2") {
            warn!("Failed to write autosuspend: {}", e);
        }
        
        fs::write(control_path, "auto")
            .context("Failed to enable charging")?;
        
        debug!("Charging enabled");
        Ok(())
    }
    
    fn disable_charging(&self, autosuspend_path: &str, control_path: &str) -> Result<()> {
        // Disable autosuspend by setting control to suspend
        // This prevents the device from drawing charging current
        fs::write(control_path, "suspend")
            .context("Failed to disable charging")?;
        
        debug!("Charging disabled");
        Ok(())
    }
    
    pub fn is_charging_enabled(&self, sys_path: &str) -> Result<bool> {
        let control_path = format!("{}/power/control", sys_path);
        
        match fs::read_to_string(&control_path) {
            Ok(content) => Ok(content.trim() == "auto"),
            Err(e) => {
                warn!("Failed to read charging state: {}", e);
                Ok(true) // Default to enabled for safety
            }
        }
    }
}


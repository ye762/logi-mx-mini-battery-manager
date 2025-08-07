// src/main.rs
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::time::Duration;
use tokio::time::sleep;

mod config;
mod hardware;
mod domain;
mod logging;

use config::Config;
use domain::BatteryManager;
use logging::setup_logging;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;
    
    let config = Config::load()?;
    info!("Starting MX Mini Battery Manager");
    
    let mut battery_manager = BatteryManager::new(config)?;
    
    loop {
        match battery_manager.check_and_manage().await {
            Ok(_) => {},
            Err(e) => error!("Error during battery check: {}", e),
        }
        
        //sleep(Duration::from_secs(60)).await;
        sleep(Duration::from_secs(10)).await;
    }
}


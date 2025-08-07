use anyhow::Result;
use log::{info, LevelFilter};

pub fn setup_logging() -> Result<()> {
    println!("Start logging setup...");
    if std::env::var("JOURNAL_STREAM").is_ok() {
        println!("Setting up journal logger...");
        // Running under systemd, use journal logger
        systemd_journal_logger::init_with_extra_fields(
            vec![("VERSION", env!("CARGO_PKG_VERSION"))]
        )?;
        log::set_max_level(LevelFilter::Info);
        info!("Journal logger initialized.");
        println!("Journal logger initialized.");
    } else {
        // Development mode, use env_logger
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
        info!("Env logger initialized.");
        println!("Env logger initialized.");
    }
    info!("Logging system ready.");
    println!("Logging setup completed. Please check logs.");
    Ok(())
}

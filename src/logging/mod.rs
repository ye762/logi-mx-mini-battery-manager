use anyhow::Result;
use log::LevelFilter;

pub fn setup_logging() -> Result<()> {
    if std::env::var("JOURNAL_STREAM").is_ok() {
        // Running under systemd, use journal logger
        systemd_journal_logger::init_with_extra_fields(
            vec![("VERSION", env!("CARGO_PKG_VERSION"))]
        )?;
    } else {
        // Development mode, use env_logger
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }
    
    log::set_max_level(LevelFilter::Info);
    log::info!("Logger successfully initialized");
    Ok(())
}

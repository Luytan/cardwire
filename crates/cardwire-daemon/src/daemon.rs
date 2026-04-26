mod config;
mod dbus;
mod models;

use crate::{config::CardwireModeState, models::Daemon};
use anyhow::{Context, Result};
use config::{CardwireConfig, CardwireGpuState};
use log::info;
use std::future::pending;
use zbus::connection;
#[tokio::main]
async fn main() -> Result<()> {
    // log
    env_logger::builder()
        .format_timestamp_nanos()
        .filter_level(log::LevelFilter::Info)
        .init();
    let config = CardwireConfig::build().context("Error building config")?;
    let gpu_state = CardwireGpuState::build().context("Error building gpu_state")?;
    let mode_state = CardwireModeState::build().context("Error building config")?;
    let mut daemon = Daemon::new(config, gpu_state, mode_state)?;
    // Now apply the config
    let _ = daemon.apply_config().await;
    let conn_builder = connection::Builder::system()?;
    let _conn = conn_builder
        .name("com.github.luytan.cardwire")?
        .serve_at("/com/github/luytan/cardwire", daemon)?
        .build()
        .await?;
    info!("Daemon started");
    pending::<()>().await;
    Ok(())
}

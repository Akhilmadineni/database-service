use anyhow::Result;
use database_service_store::StoreBootstrap;
use db_core::config::WorkerSettings;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = WorkerSettings::from_env()?;
    database_service_telemetry::init_tracing(&settings.service_name)?;

    let store = StoreBootstrap::from_env().bootstrap().await?;

    info!(
        service = %settings.service_name,
        poll_interval_secs = settings.poll_interval_secs,
        store_configured = store.is_configured(),
        store_connected = store.is_connected(),
        database_url = ?store.redacted_database_url(),
        "worker bootstrap ready"
    );

    Ok(())
}

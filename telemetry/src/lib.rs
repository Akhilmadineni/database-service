use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("{service_name}=info,axum=info,tower_http=info"))
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .compact()
                .with_target(true)
                .with_thread_ids(true),
        )
        .try_init()?;

    Ok(())
}

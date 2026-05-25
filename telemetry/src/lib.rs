use anyhow::Result;
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
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

#[derive(Default)]
struct MetricsState {
    http_requests: BTreeMap<(String, String, u16), u64>,
    reconcile_results: BTreeMap<String, u64>,
}

static METRICS: OnceLock<Mutex<MetricsState>> = OnceLock::new();

pub fn record_http_request(route: &str, method: &str, status: u16) {
    let metrics = METRICS.get_or_init(|| Mutex::new(MetricsState::default()));
    if let Ok(mut state) = metrics.lock() {
        *state
            .http_requests
            .entry((route.to_owned(), method.to_owned(), status))
            .or_insert(0) += 1;
    }
}

pub fn record_reconcile_result(state_name: &str) {
    let metrics = METRICS.get_or_init(|| Mutex::new(MetricsState::default()));
    if let Ok(mut state) = metrics.lock() {
        *state
            .reconcile_results
            .entry(state_name.to_owned())
            .or_insert(0) += 1;
    }
}

pub fn render_prometheus_metrics() -> String {
    let metrics = METRICS.get_or_init(|| Mutex::new(MetricsState::default()));
    let state = metrics
        .lock()
        .expect("metrics state lock should not be poisoned");

    let mut output = String::new();
    output.push_str("# TYPE database_service_http_requests_total counter\n");
    for ((route, method, status), count) in &state.http_requests {
        output.push_str(&format!(
            "database_service_http_requests_total{{route=\"{}\",method=\"{}\",status=\"{}\"}} {}\n",
            escape_label_value(route),
            escape_label_value(method),
            status,
            count
        ));
    }

    output.push_str("# TYPE database_service_reconcile_results_total counter\n");
    for (state_name, count) in &state.reconcile_results {
        output.push_str(&format!(
            "database_service_reconcile_results_total{{state=\"{}\"}} {}\n",
            escape_label_value(state_name),
            count
        ));
    }

    output
}

fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

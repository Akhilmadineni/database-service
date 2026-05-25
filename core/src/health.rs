use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct HealthPayload {
    pub service: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl HealthPayload {
    pub fn status(service: &str, status: &str, detail: Option<&str>) -> Self {
        Self {
            service: service.to_owned(),
            status: status.to_owned(),
            detail: detail.map(str::to_owned),
        }
    }

    pub fn ok(service: &str) -> Self {
        Self::status(service, "ok", None)
    }

    pub fn ready(service: &str, detail: &str) -> Self {
        Self::status(service, "ready", Some(detail))
    }

    pub fn bootstrap(service: &str, detail: &str) -> Self {
        Self::status(service, "bootstrap", Some(detail))
    }
}

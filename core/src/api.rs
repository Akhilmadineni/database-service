use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SuccessEnvelope<T> {
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<Uuid>,
}

impl ResponseMeta {
    pub fn new(request_id: String, operation_id: Option<Uuid>) -> Self {
        Self {
            request_id,
            operation_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorPayload,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub request_id: String,
}

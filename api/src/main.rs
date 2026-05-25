use anyhow::Result;
use axum::{
    Json, Router,
    extract::MatchedPath,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use database_service_store::{StoreBootstrap, StoreError, StoreRuntime};
use db_core::{
    api::{ErrorEnvelope, ErrorPayload, ResponseMeta, SuccessEnvelope},
    capsule::{CapsuleSpec, NormalizedCapsule, ReconcileRequest},
    config::ApiSettings,
    error::CapsuleValidationError,
    fingerprint::{desired_fingerprint, request_fingerprint},
    health::HealthPayload,
};
use serde_json::{Value, json};
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct AppState {
    service_name: String,
    store: StoreRuntime,
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = ApiSettings::from_env()?;
    database_service_telemetry::init_tracing(&settings.service_name)?;

    let store = StoreBootstrap::from_env().bootstrap().await?;
    let addr = settings.socket_addr();

    info!(
        service = %settings.service_name,
        listen_addr = %addr,
        store_configured = store.is_configured(),
        store_connected = store.is_connected(),
        database_url = ?store.redacted_database_url(),
        "starting api service"
    );

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route(
            "/v1/capsules/{app}/{environment}",
            put(put_capsule).get(get_capsule),
        )
        .route(
            "/v1/capsules/{app}/{environment}/reconcile",
            post(reconcile_capsule),
        )
        .route("/v1/operations/{operation_id}", get(get_operation))
        .layer(middleware::from_fn(http_metrics_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            service_name: settings.service_name.clone(),
            store,
        });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn healthz(State(state): State<AppState>) -> Json<HealthPayload> {
    Json(HealthPayload::ok(&state.service_name))
}

async fn metrics() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        database_service_telemetry::render_prometheus_metrics(),
    )
}

async fn readyz(State(state): State<AppState>) -> Json<HealthPayload> {
    if state.store.is_connected() {
        return Json(HealthPayload::ready(
            &state.service_name,
            "store connected and migrations complete",
        ));
    }

    let detail = if state.store.is_configured() {
        "store configured but not connected"
    } else {
        "database bootstrap pending"
    };

    Json(HealthPayload::bootstrap(&state.service_name, detail))
}

async fn put_capsule(
    State(state): State<AppState>,
    Path((app, environment)): Path<(String, String)>,
    headers: HeaderMap,
    Json(spec): Json<CapsuleSpec>,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let normalized = NormalizedCapsule::from_api(&app, &environment, &spec)
        .map_err(|error| map_capsule_validation_error(error, request_id.clone()))?;

    let idempotency_key = required_header(&headers, "Idempotency-Key", "missing_idempotency_key")
        .map_err(|error| error.with_request_id(request_id.clone()))?;
    let caller_principal = caller_principal_from_headers(&headers);
    let request_hash = request_fingerprint(&spec).map_err(|error| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            format!("failed to fingerprint request: {error}"),
            None,
            request_id.clone(),
        )
    })?;
    let desired_fingerprint = desired_fingerprint(&normalized).map_err(|error| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "fingerprint_error",
            format!("failed to fingerprint desired state: {error}"),
            None,
            request_id.clone(),
        )
    })?;

    let operation = state
        .store
        .put_capsule_request(
            &caller_principal,
            &idempotency_key,
            &request_hash,
            &spec,
            &normalized,
            &desired_fingerprint,
        )
        .await
        .map_err(|error| map_store_error(error, request_id.clone()))?;
    let operation_id = operation.operation_id;

    Ok((
        StatusCode::ACCEPTED,
        Json(SuccessEnvelope {
            data: operation,
            meta: ResponseMeta::new(request_id, Some(operation_id)),
        }),
    ))
}

async fn get_capsule(
    State(state): State<AppState>,
    Path((app, environment)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let app =
        validate_segment(&app, "app").map_err(|error| error.with_request_id(request_id.clone()))?;
    let environment = validate_segment(&environment, "environment")
        .map_err(|error| error.with_request_id(request_id.clone()))?;

    let capsule = state
        .store
        .get_capsule(&app, &environment)
        .await
        .map_err(|error| map_store_error(error, request_id.clone()))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "capsule_not_found",
                "capsule was not found",
                None,
                request_id.clone(),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(SuccessEnvelope {
            data: capsule,
            meta: ResponseMeta::new(request_id, None),
        }),
    ))
}

async fn reconcile_capsule(
    State(state): State<AppState>,
    Path((app, environment)): Path<(String, String)>,
    headers: HeaderMap,
    maybe_json: Result<Json<ReconcileRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let app =
        validate_segment(&app, "app").map_err(|error| error.with_request_id(request_id.clone()))?;
    let environment = validate_segment(&environment, "environment")
        .map_err(|error| error.with_request_id(request_id.clone()))?;
    let request = match maybe_json {
        Ok(Json(value)) => value,
        Err(_) => ReconcileRequest::default(),
    };

    let idempotency_key = required_header(&headers, "Idempotency-Key", "missing_idempotency_key")
        .map_err(|error| error.with_request_id(request_id.clone()))?;
    let caller_principal = caller_principal_from_headers(&headers);
    let request_hash = request_fingerprint(&request).map_err(|error| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            format!("failed to fingerprint request: {error}"),
            None,
            request_id.clone(),
        )
    })?;

    let operation = state
        .store
        .record_reconcile_request(
            &app,
            &environment,
            &caller_principal,
            &idempotency_key,
            &request_hash,
            &request,
        )
        .await
        .map_err(|error| map_store_error(error, request_id.clone()))?;
    let operation = state
        .store
        .process_reconcile_operation(operation.operation_id)
        .await
        .map_err(|error| map_store_error(error, request_id.clone()))?;
    database_service_telemetry::record_reconcile_result(&operation.state);
    let operation_id = operation.operation_id;

    Ok((
        StatusCode::ACCEPTED,
        Json(SuccessEnvelope {
            data: operation,
            meta: ResponseMeta::new(request_id, Some(operation_id)),
        }),
    ))
}

async fn get_operation(
    State(state): State<AppState>,
    Path(operation_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let operation = state
        .store
        .get_operation(operation_id)
        .await
        .map_err(|error| map_store_error(error, request_id.clone()))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                "operation_not_found",
                "operation was not found",
                None,
                request_id.clone(),
            )
        })?;
    let response_operation_id = operation.operation_id;

    Ok((
        StatusCode::OK,
        Json(SuccessEnvelope {
            data: operation,
            meta: ResponseMeta::new(request_id, Some(response_operation_id)),
        }),
    ))
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: String,
    message: String,
    details: Option<Value>,
    request_id: String,
}

impl ApiError {
    fn new(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
        request_id: String,
    ) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            details,
            request_id,
        }
    }

    fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let payload = ErrorEnvelope {
            error: ErrorPayload {
                code: self.code,
                message: self.message,
                details: self.details,
                request_id: self.request_id,
            },
        };

        (self.status, Json(payload)).into_response()
    }
}

fn map_store_error(error: StoreError, request_id: String) -> ApiError {
    match error {
        StoreError::NotConfigured => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "store_not_configured",
            "database store is not configured",
            None,
            request_id,
        ),
        StoreError::NotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "resource_not_found",
            "requested resource was not found",
            None,
            request_id,
        ),
        StoreError::IdempotencyConflict { operation_id } => ApiError::new(
            StatusCode::CONFLICT,
            "idempotency_conflict",
            "idempotency key was reused with a different payload",
            Some(json!({ "operation_id": operation_id })),
            request_id,
        ),
        StoreError::Sqlx(error) => ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "store_error",
            format!("database error: {error}"),
            None,
            request_id,
        ),
        StoreError::Serialization(error) => ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            format!("serialization error: {error}"),
            None,
            request_id,
        ),
        StoreError::ReconciliationFailure { message } => ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "reconciliation_failure",
            message,
            None,
            request_id,
        ),
    }
}

fn map_capsule_validation_error(error: CapsuleValidationError, request_id: String) -> ApiError {
    ApiError::new(
        StatusCode::BAD_REQUEST,
        "validation_error",
        error.to_string(),
        None,
        request_id,
    )
}

fn required_header(headers: &HeaderMap, name: &str, code: &str) -> Result<String, ApiError> {
    header_value(headers, name).ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            code,
            format!("{name} header is required"),
            None,
            String::new(),
        )
    })
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn caller_principal_from_headers(headers: &HeaderMap) -> String {
    header_value(headers, "X-Caller-Principal").unwrap_or_else(|| String::from("local-dev"))
}

fn request_id_from_headers(headers: &HeaderMap) -> String {
    header_value(headers, "X-Request-Id").unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn validate_segment(value: &str, field: &str) -> Result<String, ApiError> {
    let trimmed = value.trim().to_ascii_lowercase();
    let is_valid = (2..=50).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');

    if is_valid {
        Ok(trimmed)
    } else {
        Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "validation_error",
            format!("invalid {field}: use lowercase letters, numbers, and hyphens only"),
            None,
            String::new(),
        ))
    }
}

async fn http_metrics_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());
    let method = request.method().as_str().to_owned();
    let response = next.run(request).await;
    database_service_telemetry::record_http_request(
        &matched_path,
        &method,
        response.status().as_u16(),
    );
    response
}

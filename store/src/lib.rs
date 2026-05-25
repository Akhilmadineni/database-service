use std::env;

use db_core::{
    capsule::{CapsuleSpec, CapsuleView, ReconcileRequest},
    operation::{OperationState, OperationView},
};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction, migrate::Migrator, postgres::PgPoolOptions};
use time::OffsetDateTime;
use uuid::Uuid;

pub static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

#[derive(Clone, Debug)]
pub struct StoreBootstrap {
    database_url: Option<String>,
    pub run_migrations: bool,
    pub max_connections: u32,
}

#[derive(Clone, Debug)]
pub struct StoreRuntime {
    settings: StoreBootstrap,
    pool: Option<PgPool>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database is not configured")]
    NotConfigured,
    #[error("resource not found")]
    NotFound,
    #[error("idempotency conflict for operation {operation_id}")]
    IdempotencyConflict { operation_id: Uuid },
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, FromRow)]
struct OperationRow {
    operation_id: Uuid,
    capsule_id: Uuid,
    operation_kind: String,
    request_hash: String,
    caller_principal: String,
    state: String,
    error_code: Option<String>,
    error_message: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, FromRow)]
struct CapsuleRow {
    capsule_id: Uuid,
    app: String,
    environment: String,
    status: String,
    current_desired_fingerprint: Option<String>,
    backup_class: String,
    slo_class: String,
    current_revision_number: Option<i64>,
}

impl StoreBootstrap {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").ok();
        let run_migrations = env::var("RUN_MIGRATIONS")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(true);
        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(10);

        Self {
            database_url,
            run_migrations,
            max_connections,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.database_url.is_some()
    }

    pub fn redacted_database_url(&self) -> Option<String> {
        self.database_url
            .as_ref()
            .map(|value| redact_database_url(value))
    }

    pub async fn bootstrap(self) -> Result<StoreRuntime, sqlx::Error> {
        let pool = match &self.database_url {
            Some(database_url) => {
                let pool = PgPoolOptions::new()
                    .max_connections(self.max_connections)
                    .connect(database_url)
                    .await?;

                if self.run_migrations {
                    MIGRATOR.run(&pool).await?;
                }

                Some(pool)
            }
            None => None,
        };

        Ok(StoreRuntime {
            settings: self,
            pool,
        })
    }
}

impl StoreRuntime {
    pub fn is_configured(&self) -> bool {
        self.settings.is_configured()
    }

    pub fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    pub fn redacted_database_url(&self) -> Option<String> {
        self.settings.redacted_database_url()
    }

    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    pub async fn put_capsule_request(
        &self,
        app: &str,
        environment: &str,
        caller_principal: &str,
        idempotency_key: &str,
        request_hash: &str,
        spec: &CapsuleSpec,
    ) -> Result<OperationView, StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;
        let capsule = get_or_create_capsule(&mut transaction, app, environment, spec).await?;

        if let Some(existing) = find_existing_operation(
            &mut transaction,
            caller_principal,
            capsule.capsule_id,
            "put_capsule",
            idempotency_key,
        )
        .await?
        {
            if existing.request_hash == request_hash {
                transaction.commit().await?;
                return Ok(existing.into());
            }

            return Err(StoreError::IdempotencyConflict {
                operation_id: existing.operation_id,
            });
        }

        let desired_fingerprint = request_hash.to_owned();
        let revision_id = ensure_capsule_revision(
            &mut transaction,
            capsule.capsule_id,
            caller_principal,
            &desired_fingerprint,
            spec,
        )
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = $2,
                 current_revision_id = $3,
                 current_desired_fingerprint = $4,
                 backup_class = $5,
                 slo_class = $6,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(capsule.capsule_id)
        .bind("requested")
        .bind(revision_id)
        .bind(&desired_fingerprint)
        .bind(&spec.protection_classes.backup_class)
        .bind(&spec.protection_classes.slo_class)
        .execute(transaction.as_mut())
        .await?;

        let operation_id = Uuid::new_v4();
        let operation = sqlx::query_as::<_, OperationRow>(
            "INSERT INTO operation (
                 operation_id,
                 capsule_id,
                 operation_kind,
                 idempotency_key,
                 request_hash,
                 caller_principal,
                 state
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING
                 operation_id,
                 capsule_id,
                 operation_kind,
                 request_hash,
                 caller_principal,
                 state,
                 error_code,
                 error_message,
                 created_at,
                 updated_at,
                 completed_at",
        )
        .bind(operation_id)
        .bind(capsule.capsule_id)
        .bind("put_capsule")
        .bind(idempotency_key)
        .bind(request_hash)
        .bind(caller_principal)
        .bind(OperationState::Accepted.as_str())
        .fetch_one(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(operation.into())
    }

    pub async fn record_reconcile_request(
        &self,
        app: &str,
        environment: &str,
        caller_principal: &str,
        idempotency_key: &str,
        request_hash: &str,
        _request: &ReconcileRequest,
    ) -> Result<OperationView, StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;
        let capsule_id = sqlx::query_scalar!(
            "SELECT capsule_id
             FROM capsule
             WHERE app = $1 AND environment = $2
             FOR UPDATE",
            app,
            environment
        )
        .fetch_optional(transaction.as_mut())
        .await?
        .ok_or(StoreError::NotFound)?;

        if let Some(existing) = find_existing_operation(
            &mut transaction,
            caller_principal,
            capsule_id,
            "reconcile",
            idempotency_key,
        )
        .await?
        {
            if existing.request_hash == request_hash {
                transaction.commit().await?;
                return Ok(existing.into());
            }

            return Err(StoreError::IdempotencyConflict {
                operation_id: existing.operation_id,
            });
        }

        let operation_id = Uuid::new_v4();
        let operation = sqlx::query_as::<_, OperationRow>(
            "INSERT INTO operation (
                 operation_id,
                 capsule_id,
                 operation_kind,
                 idempotency_key,
                 request_hash,
                 caller_principal,
                 state
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING
                 operation_id,
                 capsule_id,
                 operation_kind,
                 request_hash,
                 caller_principal,
                 state,
                 error_code,
                 error_message,
                 created_at,
                 updated_at,
                 completed_at",
        )
        .bind(operation_id)
        .bind(capsule_id)
        .bind("reconcile")
        .bind(idempotency_key)
        .bind(request_hash)
        .bind(caller_principal)
        .bind(OperationState::Accepted.as_str())
        .fetch_one(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(operation.into())
    }

    pub async fn get_operation(
        &self,
        operation_id: Uuid,
    ) -> Result<Option<OperationView>, StoreError> {
        let pool = self.require_pool()?;
        let operation = sqlx::query_as!(
            OperationRow,
            "SELECT
                 operation_id,
                 capsule_id,
                 operation_kind,
                 request_hash,
                 caller_principal,
                 state,
                 error_code,
                 error_message,
                 created_at,
                 updated_at,
                 completed_at
             FROM operation
             WHERE operation_id = $1",
            operation_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(operation.map(Into::into))
    }

    pub async fn get_capsule(
        &self,
        app: &str,
        environment: &str,
    ) -> Result<Option<CapsuleView>, StoreError> {
        let pool = self.require_pool()?;
        let capsule = sqlx::query_as!(
            CapsuleRow,
            "SELECT
                 c.capsule_id,
                 c.app,
                 c.environment,
                 c.status,
                 c.current_desired_fingerprint,
                 c.backup_class,
                 c.slo_class,
                 cr.revision_number AS \"current_revision_number?\"
             FROM capsule c
             LEFT JOIN capsule_revision cr
               ON cr.capsule_revision_id = c.current_revision_id
             WHERE c.app = $1 AND c.environment = $2",
            app,
            environment
        )
        .fetch_optional(pool)
        .await?;

        Ok(capsule.map(Into::into))
    }

    fn require_pool(&self) -> Result<&PgPool, StoreError> {
        self.pool.as_ref().ok_or(StoreError::NotConfigured)
    }
}

async fn get_or_create_capsule(
    transaction: &mut Transaction<'_, Postgres>,
    app: &str,
    environment: &str,
    spec: &CapsuleSpec,
) -> Result<CapsuleRow, StoreError> {
    if let Some(existing) = sqlx::query_as!(
        CapsuleRow,
        "SELECT
             capsule_id,
             app,
             environment,
             status,
             current_desired_fingerprint,
             backup_class,
             slo_class,
             NULL::bigint AS \"current_revision_number?\"
         FROM capsule
         WHERE app = $1 AND environment = $2
         FOR UPDATE",
        app,
        environment
    )
    .fetch_optional(transaction.as_mut())
    .await?
    {
        return Ok(existing);
    }

    let capsule_id = Uuid::new_v4();
    let row = sqlx::query_as!(
        CapsuleRow,
        "INSERT INTO capsule (
             capsule_id,
             app,
             environment,
             status,
             backup_class,
             slo_class
         )
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING
             capsule_id,
             app,
             environment,
             status,
             current_desired_fingerprint,
             backup_class,
             slo_class,
             NULL::bigint AS \"current_revision_number?\"",
        capsule_id,
        app,
        environment,
        "requested",
        spec.protection_classes.backup_class,
        spec.protection_classes.slo_class
    )
    .fetch_one(transaction.as_mut())
    .await?;

    Ok(row)
}

async fn ensure_capsule_revision(
    transaction: &mut Transaction<'_, Postgres>,
    capsule_id: Uuid,
    caller_principal: &str,
    desired_fingerprint: &str,
    spec: &CapsuleSpec,
) -> Result<Uuid, StoreError> {
    let existing_revision_id = sqlx::query_scalar!(
        "SELECT capsule_revision_id
         FROM capsule_revision
         WHERE capsule_id = $1 AND desired_fingerprint = $2",
        capsule_id,
        desired_fingerprint
    )
    .fetch_optional(transaction.as_mut())
    .await?;

    if let Some(revision_id) = existing_revision_id {
        return Ok(revision_id);
    }

    let next_revision_number = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(revision_number), 0)::bigint + 1
         FROM capsule_revision
         WHERE capsule_id = $1",
        capsule_id
    )
    .fetch_one(transaction.as_mut())
    .await?;

    let revision_id = Uuid::new_v4();
    let spec_json = serde_json::to_value(spec)?;
    let normalized_spec_json = normalize_spec(spec)?;

    sqlx::query(
        "INSERT INTO capsule_revision (
             capsule_revision_id,
             capsule_id,
             revision_number,
             spec_json,
             normalized_spec_json,
             desired_fingerprint,
             created_by_principal
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(revision_id)
    .bind(capsule_id)
    .bind(next_revision_number)
    .bind(spec_json)
    .bind(normalized_spec_json)
    .bind(desired_fingerprint)
    .bind(caller_principal)
    .execute(transaction.as_mut())
    .await?;

    Ok(revision_id)
}

async fn find_existing_operation(
    transaction: &mut Transaction<'_, Postgres>,
    caller_principal: &str,
    capsule_id: Uuid,
    operation_kind: &str,
    idempotency_key: &str,
) -> Result<Option<OperationRow>, StoreError> {
    let operation = sqlx::query_as!(
        OperationRow,
        "SELECT
             operation_id,
             capsule_id,
             operation_kind,
             request_hash,
             caller_principal,
             state,
             error_code,
             error_message,
             created_at,
             updated_at,
             completed_at
         FROM operation
         WHERE caller_principal = $1
           AND capsule_id = $2
           AND operation_kind = $3
           AND idempotency_key = $4",
        caller_principal,
        capsule_id,
        operation_kind,
        idempotency_key
    )
    .fetch_optional(transaction.as_mut())
    .await?;

    Ok(operation)
}

fn normalize_spec(spec: &CapsuleSpec) -> Result<Value, serde_json::Error> {
    let mut value = serde_json::to_value(spec)?;
    sort_json_value(&mut value);
    Ok(value)
}

fn sort_json_value(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for item in values.iter_mut() {
                sort_json_value(item);
            }

            values.sort_by_key(|left| left.to_string());
        }
        Value::Object(map) => {
            let mut entries = map
                .iter_mut()
                .map(|(key, value)| {
                    sort_json_value(value);
                    (key.clone(), value.clone())
                })
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            map.clear();
            for (key, value) in entries {
                map.insert(key, value);
            }
        }
        _ => {}
    }
}

fn redact_database_url(input: &str) -> String {
    match input.rsplit_once('@') {
        Some((prefix, suffix)) if prefix.contains("://") => {
            let scheme = prefix
                .split_once("://")
                .map(|(scheme, _)| scheme)
                .unwrap_or("postgres");
            format!("{scheme}://***:***@{suffix}")
        }
        _ => String::from("***"),
    }
}

impl From<OperationRow> for OperationView {
    fn from(value: OperationRow) -> Self {
        Self {
            operation_id: value.operation_id,
            capsule_id: value.capsule_id,
            operation_kind: value.operation_kind,
            request_hash: value.request_hash,
            caller_principal: value.caller_principal,
            state: value.state,
            error_code: value.error_code,
            error_message: value.error_message,
            created_at: value.created_at,
            updated_at: value.updated_at,
            completed_at: value.completed_at,
        }
    }
}

impl From<CapsuleRow> for CapsuleView {
    fn from(value: CapsuleRow) -> Self {
        Self {
            capsule_id: value.capsule_id,
            app: value.app,
            environment: value.environment,
            status: value.status,
            current_revision_number: value.current_revision_number,
            current_desired_fingerprint: value.current_desired_fingerprint,
            backup_class: value.backup_class,
            slo_class: value.slo_class,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StoreBootstrap, StoreError, redact_database_url};
    use db_core::capsule::{
        CapsuleSpec, DatabaseSpec, PrivilegePolicy, ProtectionClasses, ReconcileRequest, RoleSpec,
        SafetyPolicy,
    };
    use sqlx::Row;
    use uuid::Uuid;

    #[test]
    fn redacts_credentials_in_database_url() {
        let input = "postgresql://user:secret@localhost:5432/control";
        assert_eq!(
            redact_database_url(input),
            "postgresql://***:***@localhost:5432/control"
        );
    }

    #[tokio::test]
    async fn runs_migrations_when_database_url_is_present() {
        let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
            return;
        };

        let runtime = StoreBootstrap {
            database_url: Some(database_url),
            run_migrations: true,
            max_connections: 2,
        }
        .bootstrap()
        .await
        .expect("store bootstrap should succeed");

        let pool = runtime.pool().expect("database pool should exist");
        let row = sqlx::query(
            "SELECT COUNT(*) AS table_count
             FROM information_schema.tables
             WHERE table_schema = 'public'
               AND table_name IN (
                 'capsule',
                 'capsule_revision',
                 'operation',
                 'reconciliation_run',
                 'checkpoint',
                 'audit_event'
               )",
        )
        .fetch_one(pool)
        .await
        .expect("table count query should succeed");

        let table_count: i64 = row.get("table_count");
        assert_eq!(table_count, 6);
    }

    #[tokio::test]
    async fn put_capsule_request_is_idempotent_when_hash_matches() {
        let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
            return;
        };

        let runtime = StoreBootstrap {
            database_url: Some(database_url),
            run_migrations: true,
            max_connections: 2,
        }
        .bootstrap()
        .await
        .expect("store bootstrap should succeed");

        let app = format!("app-{}", Uuid::new_v4().simple());
        let environment = format!("env-{}", Uuid::new_v4().simple());
        let spec = sample_spec(&app, &environment);

        let first = runtime
            .put_capsule_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-1",
                "hash-1",
                &spec,
            )
            .await
            .expect("first request should succeed");

        let second = runtime
            .put_capsule_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-1",
                "hash-1",
                &spec,
            )
            .await
            .expect("second request should resolve to same operation");

        assert_eq!(first.operation_id, second.operation_id);
    }

    #[tokio::test]
    async fn put_capsule_request_conflicts_when_hash_differs() {
        let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
            return;
        };

        let runtime = StoreBootstrap {
            database_url: Some(database_url),
            run_migrations: true,
            max_connections: 2,
        }
        .bootstrap()
        .await
        .expect("store bootstrap should succeed");

        let app = format!("app-{}", Uuid::new_v4().simple());
        let environment = format!("env-{}", Uuid::new_v4().simple());
        let spec = sample_spec(&app, &environment);

        let first = runtime
            .put_capsule_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-2",
                "hash-a",
                &spec,
            )
            .await
            .expect("first request should succeed");

        let error = runtime
            .put_capsule_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-2",
                "hash-b",
                &spec,
            )
            .await
            .expect_err("second request should conflict");

        match error {
            StoreError::IdempotencyConflict { operation_id } => {
                assert_eq!(operation_id, first.operation_id);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn reconcile_request_creates_operation_for_existing_capsule() {
        let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
            return;
        };

        let runtime = StoreBootstrap {
            database_url: Some(database_url),
            run_migrations: true,
            max_connections: 2,
        }
        .bootstrap()
        .await
        .expect("store bootstrap should succeed");

        let app = format!("app-{}", Uuid::new_v4().simple());
        let environment = format!("env-{}", Uuid::new_v4().simple());
        let spec = sample_spec(&app, &environment);

        runtime
            .put_capsule_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-3",
                "hash-c",
                &spec,
            )
            .await
            .expect("capsule request should succeed");

        let reconcile = runtime
            .record_reconcile_request(
                &app,
                &environment,
                "test-principal",
                "idem-key-4",
                "hash-d",
                &ReconcileRequest {
                    reason: Some(String::from("test")),
                },
            )
            .await
            .expect("reconcile request should succeed");

        assert_eq!(reconcile.operation_kind, "reconcile");
        assert_eq!(reconcile.state, "accepted");
    }

    fn sample_spec(app: &str, environment: &str) -> CapsuleSpec {
        CapsuleSpec {
            database: DatabaseSpec {
                name: format!("app_{app}_{environment}"),
                owner_role: format!("{app}_{environment}_owner"),
            },
            roles: vec![RoleSpec {
                name: format!("{app}_{environment}_app"),
                connection_limit: Some(50),
            }],
            privilege_policy: PrivilegePolicy {
                managed_schemas: vec![String::from("public")],
            },
            protection_classes: ProtectionClasses {
                backup_class: String::from("standard"),
                slo_class: String::from("gold"),
            },
            safety_policy: SafetyPolicy {
                allow_destructive_mutation: false,
            },
        }
    }
}

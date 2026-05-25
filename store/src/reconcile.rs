use db_core::{
    capsule::NormalizedCapsule,
    drift::{DriftReport, DriftSeverity, classify_drift},
    fingerprint::{checkpoint_fingerprint, observed_fingerprint, plan_fingerprint},
    observed::{
        ObservedCapsuleState, ObservedDatabaseState, ObservedRoleState, ObservedRuntimeState,
        ObservedSchemaState,
    },
    operation::{OperationState, OperationView},
    plan::{ReconciliationPlan, StageKind, plan_from_drift},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, Row};
use url::Url;
use uuid::Uuid;

use super::*;

#[derive(Clone, Debug, Default)]
struct ReconcileExecutionOptions {
    fail_after_stage: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ReconciliationCheckpoint {
    database: DatabaseCheckpoint,
    roles: Vec<RoleCheckpoint>,
    schemas: Vec<SchemaCheckpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DatabaseCheckpoint {
    name: String,
    existed: bool,
    owner_before: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RoleCheckpoint {
    name: String,
    existed: bool,
    connection_limit_before: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SchemaCheckpoint {
    name: String,
    existed: bool,
}

#[derive(Clone, Debug)]
struct ReconcileContext {
    operation_id: Uuid,
    capsule_id: Uuid,
    prior_capsule_state: String,
    normalized_capsule: NormalizedCapsule,
    desired_fingerprint: String,
}

impl StoreRuntime {
    pub async fn observe_target_state(
        &self,
        desired: &NormalizedCapsule,
    ) -> Result<ObservedCapsuleState, StoreError> {
        let mut admin = self.admin_connection().await?;
        let database_row = sqlx::query(
            "SELECT d.datname, r.rolname AS owner_role
             FROM pg_database d
             LEFT JOIN pg_roles r ON r.oid = d.datdba
             WHERE d.datname = $1",
        )
        .bind(&desired.database.name)
        .fetch_optional(&mut admin)
        .await?;

        let database = match database_row {
            Some(row) => ObservedDatabaseState {
                name: row.get::<String, _>("datname"),
                exists: true,
                owner_role: row.get::<Option<String>, _>("owner_role"),
            },
            None => ObservedDatabaseState {
                name: desired.database.name.clone(),
                exists: false,
                owner_role: None,
            },
        };

        let mut roles = Vec::with_capacity(desired.roles.len());
        for role in &desired.roles {
            let row = sqlx::query(
                "SELECT rolname, rolconnlimit
                 FROM pg_roles
                 WHERE rolname = $1",
            )
            .bind(&role.name)
            .fetch_optional(&mut admin)
            .await?;

            roles.push(match row {
                Some(row) => ObservedRoleState {
                    name: row.get::<String, _>("rolname"),
                    exists: true,
                    connection_limit: normalize_connection_limit(row.get::<i32, _>("rolconnlimit")),
                },
                None => ObservedRoleState {
                    name: role.name.clone(),
                    exists: false,
                    connection_limit: None,
                },
            });
        }
        roles.sort();

        let runtime = if database.exists {
            ObservedRuntimeState {
                active_session_count: sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*)
                     FROM pg_stat_activity
                     WHERE datname = $1
                       AND pid <> pg_backend_pid()",
                )
                .bind(&desired.database.name)
                .fetch_one(&mut admin)
                .await?
                .max(0) as u32,
                active_lock_count: sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*)
                     FROM pg_locks l
                     JOIN pg_database d ON d.oid = l.database
                     WHERE d.datname = $1",
                )
                .bind(&desired.database.name)
                .fetch_one(&mut admin)
                .await?
                .max(0) as u32,
            }
        } else {
            ObservedRuntimeState::default()
        };

        let mut schemas = Vec::new();
        if database.exists {
            let mut target = self
                .target_database_connection(&desired.database.name)
                .await?;
            for schema_name in &desired.privilege_policy.managed_schemas {
                let exists = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*)
                     FROM pg_namespace
                     WHERE nspname = $1",
                )
                .bind(schema_name)
                .fetch_one(&mut target)
                .await?
                    > 0;

                schemas.push(ObservedSchemaState {
                    name: schema_name.clone(),
                    exists,
                });
            }
            schemas.sort();
        }

        Ok(ObservedCapsuleState {
            app: desired.identity.app.clone(),
            environment: desired.identity.environment.clone(),
            database,
            roles,
            schemas,
            runtime,
        })
    }

    pub async fn process_reconcile_operation(
        &self,
        operation_id: Uuid,
    ) -> Result<OperationView, StoreError> {
        self.process_reconcile_operation_with_options(
            operation_id,
            ReconcileExecutionOptions::default(),
        )
        .await
    }

    async fn process_reconcile_operation_with_options(
        &self,
        operation_id: Uuid,
        options: ReconcileExecutionOptions,
    ) -> Result<OperationView, StoreError> {
        let context = self.load_reconcile_context(operation_id).await?;
        self.transition_operation_and_capsule(
            context.operation_id,
            context.capsule_id,
            OperationState::InProgress,
            "reconciling",
            None,
            None,
        )
        .await?;

        let observed_before = self
            .observe_target_state(&context.normalized_capsule)
            .await?;
        let observed_before_fingerprint = observed_fingerprint(&observed_before)?;
        let drift = classify_drift(&context.normalized_capsule, &observed_before);
        let plan = plan_from_drift(&drift);
        let plan_fingerprint = plan_fingerprint(&plan)?;
        let run_id = self
            .insert_reconciliation_run(
                &context,
                &drift,
                &observed_before_fingerprint,
                &plan_fingerprint,
            )
            .await?;
        self.record_audit_event(
            context.capsule_id,
            Some(context.operation_id),
            Some(run_id),
            "observation_captured",
            json!({
                "observed_fingerprint": observed_before_fingerprint,
                "active_session_count": observed_before.runtime.active_session_count,
                "active_lock_count": observed_before.runtime.active_lock_count,
            }),
        )
        .await?;
        self.record_audit_event(
            context.capsule_id,
            Some(context.operation_id),
            Some(run_id),
            "drift_classified",
            serde_json::to_value(&drift)?,
        )
        .await?;
        self.record_audit_event(
            context.capsule_id,
            Some(context.operation_id),
            Some(run_id),
            "plan_generated",
            serde_json::to_value(&plan)?,
        )
        .await?;

        if !drift.requires_reconcile {
            self.complete_reconciliation_success(
                &context,
                run_id,
                &observed_before_fingerprint,
                &plan_fingerprint,
            )
            .await?;
            return self
                .get_operation(operation_id)
                .await?
                .ok_or(StoreError::NotFound);
        }

        if plan.blocks_autonomous_apply {
            self.complete_reconciliation_blocked(
                &context,
                run_id,
                &drift,
                &observed_before_fingerprint,
                &plan_fingerprint,
            )
            .await?;
            return self
                .get_operation(operation_id)
                .await?
                .ok_or(StoreError::NotFound);
        }

        let checkpoint = capture_checkpoint(&context.normalized_capsule, &observed_before);
        let checkpoint_fingerprint = checkpoint_fingerprint(&checkpoint)?;
        self.insert_checkpoint(run_id, &checkpoint, &checkpoint_fingerprint)
            .await?;
        self.record_audit_event(
            context.capsule_id,
            Some(context.operation_id),
            Some(run_id),
            "checkpoint_written",
            json!({ "checkpoint_fingerprint": checkpoint_fingerprint }),
        )
        .await?;

        let apply_result = self
            .apply_plan(
                &context.normalized_capsule,
                &observed_before,
                &plan,
                &checkpoint,
                &options,
            )
            .await;

        if let Err(error) = apply_result {
            self.record_audit_event(
                context.capsule_id,
                Some(context.operation_id),
                Some(run_id),
                "rollback_started",
                json!({ "error": error.to_string() }),
            )
            .await?;
            let rollback = self
                .rollback_from_checkpoint(&context.normalized_capsule, &checkpoint)
                .await;
            match rollback {
                Ok(()) => {
                    self.record_audit_event(
                        context.capsule_id,
                        Some(context.operation_id),
                        Some(run_id),
                        "rollback_completed",
                        json!({ "outcome": "completed" }),
                    )
                    .await?;
                    self.complete_reconciliation_failed(
                        &context,
                        run_id,
                        &observed_before_fingerprint,
                        &plan_fingerprint,
                        Some(checkpoint_fingerprint),
                        Some(String::from("completed")),
                        format!("reconciliation failed and rollback completed: {error}"),
                        "requested",
                    )
                    .await?;
                }
                Err(rollback_error) => {
                    self.record_audit_event(
                        context.capsule_id,
                        Some(context.operation_id),
                        Some(run_id),
                        "rollback_completed",
                        json!({ "outcome": "failed", "error": rollback_error.to_string() }),
                    )
                    .await?;
                    self.complete_reconciliation_failed(
                        &context,
                        run_id,
                        &observed_before_fingerprint,
                        &plan_fingerprint,
                        Some(checkpoint_fingerprint),
                        Some(String::from("failed")),
                        format!("reconciliation failed and rollback failed: {rollback_error}"),
                        "manual_intervention",
                    )
                    .await?;
                }
            }

            return self
                .get_operation(operation_id)
                .await?
                .ok_or(StoreError::NotFound);
        }

        let observed_after = self
            .observe_target_state(&context.normalized_capsule)
            .await?;
        let observed_after_fingerprint = observed_fingerprint(&observed_after)?;
        let drift_after = classify_drift(&context.normalized_capsule, &observed_after);
        if drift_after.requires_reconcile {
            self.record_audit_event(
                context.capsule_id,
                Some(context.operation_id),
                Some(run_id),
                "rollback_started",
                json!({ "error": "post-apply verification failed" }),
            )
            .await?;
            let rollback = self
                .rollback_from_checkpoint(&context.normalized_capsule, &checkpoint)
                .await;
            let rollback_outcome = if rollback.is_ok() {
                String::from("completed")
            } else {
                String::from("failed")
            };
            self.complete_reconciliation_failed(
                &context,
                run_id,
                &observed_after_fingerprint,
                &plan_fingerprint,
                Some(checkpoint_fingerprint),
                Some(rollback_outcome.clone()),
                String::from("post-apply verification failed"),
                if rollback.is_ok() {
                    "requested"
                } else {
                    "manual_intervention"
                },
            )
            .await?;
            return self
                .get_operation(operation_id)
                .await?
                .ok_or(StoreError::NotFound);
        }

        self.complete_reconciliation_verified(
            &context,
            run_id,
            &observed_after_fingerprint,
            &plan_fingerprint,
            Some(checkpoint_fingerprint),
        )
        .await?;
        self.record_audit_event(
            context.capsule_id,
            Some(context.operation_id),
            Some(run_id),
            "verification_passed",
            json!({ "observed_fingerprint_after": observed_after_fingerprint }),
        )
        .await?;

        self.get_operation(operation_id)
            .await?
            .ok_or(StoreError::NotFound)
    }
}

impl StoreRuntime {
    async fn load_reconcile_context(
        &self,
        operation_id: Uuid,
    ) -> Result<ReconcileContext, StoreError> {
        let pool = self.require_pool()?;
        let row = sqlx::query(
            "SELECT
                 o.operation_id,
                 o.capsule_id,
                 c.status,
                 cr.normalized_spec_json,
                 cr.desired_fingerprint
             FROM operation o
             JOIN capsule c ON c.capsule_id = o.capsule_id
             JOIN capsule_revision cr ON cr.capsule_revision_id = c.current_revision_id
             WHERE o.operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(pool)
        .await?
        .ok_or(StoreError::NotFound)?;

        let normalized_capsule = serde_json::from_value::<NormalizedCapsule>(
            row.get::<serde_json::Value, _>("normalized_spec_json"),
        )?;

        Ok(ReconcileContext {
            operation_id: row.get("operation_id"),
            capsule_id: row.get("capsule_id"),
            prior_capsule_state: row.get("status"),
            normalized_capsule,
            desired_fingerprint: row.get("desired_fingerprint"),
        })
    }

    async fn transition_operation_and_capsule(
        &self,
        operation_id: Uuid,
        capsule_id: Uuid,
        state: OperationState,
        capsule_status: &str,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "UPDATE operation
             SET state = $2,
                 error_code = $3,
                 error_message = $4,
                 completed_at = CASE
                     WHEN $2 IN ('succeeded', 'blocked', 'failed') THEN now()
                     ELSE completed_at
                 END,
                 updated_at = now()
             WHERE operation_id = $1",
        )
        .bind(operation_id)
        .bind(state.as_str())
        .bind(error_code)
        .bind(error_message)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = $2,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(capsule_id)
        .bind(capsule_status)
        .execute(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn insert_reconciliation_run(
        &self,
        context: &ReconcileContext,
        drift: &DriftReport,
        observed_before_fingerprint: &str,
        plan_fingerprint: &str,
    ) -> Result<Uuid, StoreError> {
        let pool = self.require_pool()?;
        let run_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO reconciliation_run (
                 reconciliation_run_id,
                 operation_id,
                 capsule_id,
                 capsule_revision_id,
                 prior_capsule_state,
                 drift_classification,
                 desired_fingerprint,
                 observed_fingerprint_before,
                 plan_fingerprint,
                 rollback_required
             )
             SELECT
                 $1,
                 $2,
                 c.capsule_id,
                 c.current_revision_id,
                 $3,
                 $4,
                 $5,
                 $6,
                 $7,
                 $8
             FROM capsule c
             WHERE c.capsule_id = $9",
        )
        .bind(run_id)
        .bind(context.operation_id)
        .bind(&context.prior_capsule_state)
        .bind(drift_label(drift))
        .bind(&context.desired_fingerprint)
        .bind(observed_before_fingerprint)
        .bind(plan_fingerprint)
        .bind(!drift.items.is_empty())
        .bind(context.capsule_id)
        .execute(pool)
        .await?;

        Ok(run_id)
    }

    async fn complete_reconciliation_success(
        &self,
        context: &ReconcileContext,
        run_id: Uuid,
        observed_fingerprint: &str,
        plan_fingerprint_value: &str,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "UPDATE reconciliation_run
             SET final_capsule_state = $2,
                 observed_fingerprint_after = $3,
                 completed_at = now()
             WHERE reconciliation_run_id = $1",
        )
        .bind(run_id)
        .bind("converged")
        .bind(observed_fingerprint)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = 'converged',
                 last_observed_fingerprint = $2,
                 last_plan_fingerprint = $3,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(context.capsule_id)
        .bind(observed_fingerprint)
        .bind(plan_fingerprint_value)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE operation
             SET state = 'succeeded',
                 updated_at = now(),
                 completed_at = now()
             WHERE operation_id = $1",
        )
        .bind(context.operation_id)
        .execute(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn complete_reconciliation_blocked(
        &self,
        context: &ReconcileContext,
        run_id: Uuid,
        drift: &DriftReport,
        observed_fingerprint: &str,
        plan_fingerprint_value: &str,
    ) -> Result<(), StoreError> {
        let message = drift
            .items
            .iter()
            .map(|item| item.message.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "UPDATE reconciliation_run
             SET final_capsule_state = 'blocked_unsafe',
                 observed_fingerprint_after = $2,
                 completed_at = now()
             WHERE reconciliation_run_id = $1",
        )
        .bind(run_id)
        .bind(observed_fingerprint)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = 'blocked_unsafe',
                 last_observed_fingerprint = $2,
                 last_plan_fingerprint = $3,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(context.capsule_id)
        .bind(observed_fingerprint)
        .bind(plan_fingerprint_value)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE operation
             SET state = 'blocked',
                 error_code = 'blocked_unsafe',
                 error_message = $2,
                 updated_at = now(),
                 completed_at = now()
             WHERE operation_id = $1",
        )
        .bind(context.operation_id)
        .bind(message)
        .execute(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn complete_reconciliation_failed(
        &self,
        context: &ReconcileContext,
        run_id: Uuid,
        observed_fingerprint_after: &str,
        plan_fingerprint_value: &str,
        checkpoint_fingerprint_value: Option<String>,
        rollback_outcome: Option<String>,
        error_message: String,
        capsule_status: &str,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "UPDATE reconciliation_run
             SET final_capsule_state = $2,
                 observed_fingerprint_after = $3,
                 checkpoint_fingerprint = COALESCE($4, checkpoint_fingerprint),
                 rollback_outcome = $5,
                 completed_at = now()
             WHERE reconciliation_run_id = $1",
        )
        .bind(run_id)
        .bind(capsule_status)
        .bind(observed_fingerprint_after)
        .bind(checkpoint_fingerprint_value)
        .bind(rollback_outcome)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = $2,
                 last_plan_fingerprint = $3,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(context.capsule_id)
        .bind(capsule_status)
        .bind(plan_fingerprint_value)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE operation
             SET state = 'failed',
                 error_code = 'reconciliation_failed',
                 error_message = $2,
                 updated_at = now(),
                 completed_at = now()
             WHERE operation_id = $1",
        )
        .bind(context.operation_id)
        .bind(error_message)
        .execute(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn complete_reconciliation_verified(
        &self,
        context: &ReconcileContext,
        run_id: Uuid,
        observed_fingerprint_after: &str,
        plan_fingerprint_value: &str,
        checkpoint_fingerprint_value: Option<String>,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        let mut transaction = pool.begin().await?;

        sqlx::query(
            "UPDATE reconciliation_run
             SET final_capsule_state = 'converged',
                 observed_fingerprint_after = $2,
                 checkpoint_fingerprint = COALESCE($3, checkpoint_fingerprint),
                 rollback_outcome = 'not_needed',
                 completed_at = now()
             WHERE reconciliation_run_id = $1",
        )
        .bind(run_id)
        .bind(observed_fingerprint_after)
        .bind(checkpoint_fingerprint_value)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE capsule
             SET status = 'converged',
                 last_observed_fingerprint = $2,
                 last_plan_fingerprint = $3,
                 updated_at = now()
             WHERE capsule_id = $1",
        )
        .bind(context.capsule_id)
        .bind(observed_fingerprint_after)
        .bind(plan_fingerprint_value)
        .execute(transaction.as_mut())
        .await?;

        sqlx::query(
            "UPDATE operation
             SET state = 'succeeded',
                 updated_at = now(),
                 completed_at = now()
             WHERE operation_id = $1",
        )
        .bind(context.operation_id)
        .execute(transaction.as_mut())
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn insert_checkpoint(
        &self,
        run_id: Uuid,
        checkpoint: &ReconciliationCheckpoint,
        checkpoint_fingerprint_value: &str,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        sqlx::query(
            "INSERT INTO checkpoint (
                 checkpoint_id,
                 reconciliation_run_id,
                 checkpoint_json,
                 checkpoint_fingerprint
             )
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(serde_json::to_value(checkpoint)?)
        .bind(checkpoint_fingerprint_value)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn record_audit_event(
        &self,
        capsule_id: Uuid,
        operation_id: Option<Uuid>,
        run_id: Option<Uuid>,
        event_type: &str,
        payload_json: serde_json::Value,
    ) -> Result<(), StoreError> {
        let pool = self.require_pool()?;
        let previous_hash = sqlx::query_scalar::<_, Option<Vec<u8>>>(
            "SELECT event_hash
             FROM audit_event
             WHERE capsule_id = $1
             ORDER BY occurred_at DESC
             LIMIT 1",
        )
        .bind(capsule_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        let event_hash = compute_event_hash(
            previous_hash.as_deref(),
            event_type,
            &payload_json,
            operation_id,
            run_id,
        )?;

        sqlx::query(
            "INSERT INTO audit_event (
                 audit_event_id,
                 capsule_id,
                 operation_id,
                 reconciliation_run_id,
                 event_type,
                 payload_json,
                 prev_hash,
                 event_hash
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(Uuid::new_v4())
        .bind(capsule_id)
        .bind(operation_id)
        .bind(run_id)
        .bind(event_type)
        .bind(payload_json)
        .bind(previous_hash)
        .bind(event_hash)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn apply_plan(
        &self,
        desired: &NormalizedCapsule,
        observed_before: &ObservedCapsuleState,
        plan: &ReconciliationPlan,
        _checkpoint: &ReconciliationCheckpoint,
        options: &ReconcileExecutionOptions,
    ) -> Result<(), StoreError> {
        for (index, stage) in plan.stages.iter().enumerate() {
            match stage.kind {
                StageKind::EnsureRole => {
                    for role in &desired.roles {
                        self.ensure_role(role.name.as_str(), role.connection_limit)
                            .await?;
                    }
                }
                StageKind::SetRoleConnectionLimit => {
                    for role in &desired.roles {
                        self.set_role_connection_limit(role.name.as_str(), role.connection_limit)
                            .await?;
                    }
                }
                StageKind::EnsureDatabase => {
                    self.ensure_database(
                        desired.database.name.as_str(),
                        desired.database.owner_role.as_str(),
                    )
                    .await?;
                }
                StageKind::SetDatabaseOwner => {
                    if observed_before.database.exists {
                        self.set_database_owner(
                            desired.database.name.as_str(),
                            desired.database.owner_role.as_str(),
                        )
                        .await?;
                    }
                }
                StageKind::EnsureSchema => {
                    for schema in &desired.privilege_policy.managed_schemas {
                        if schema != "public" {
                            self.ensure_schema(
                                desired.database.name.as_str(),
                                schema.as_str(),
                                desired.database.owner_role.as_str(),
                            )
                            .await?;
                        }
                    }
                }
            }

            if options.fail_after_stage == Some(index + 1) {
                return Err(StoreError::ReconciliationFailure {
                    message: format!("failpoint triggered after stage {:?}", stage.kind),
                });
            }
        }

        Ok(())
    }

    async fn rollback_from_checkpoint(
        &self,
        desired: &NormalizedCapsule,
        checkpoint: &ReconciliationCheckpoint,
    ) -> Result<(), StoreError> {
        if !checkpoint.database.existed {
            self.drop_database(desired.database.name.as_str()).await?;
        } else if let Some(owner_before) = &checkpoint.database.owner_before {
            self.set_database_owner(desired.database.name.as_str(), owner_before)
                .await?;
        }

        if checkpoint.database.existed {
            for schema in checkpoint.schemas.iter().rev() {
                if !schema.existed && schema.name != "public" {
                    self.drop_schema(desired.database.name.as_str(), schema.name.as_str())
                        .await?;
                }
            }
        }

        for role in checkpoint.roles.iter().rev() {
            if !role.existed {
                self.drop_role(role.name.as_str()).await?;
            } else {
                self.set_role_connection_limit(role.name.as_str(), role.connection_limit_before)
                    .await?;
            }
        }

        Ok(())
    }

    async fn admin_connection(&self) -> Result<PgConnection, StoreError> {
        let database_url = self.target_admin_database_url()?.to_owned();
        Ok(PgConnection::connect(&database_url).await?)
    }

    async fn target_database_connection(
        &self,
        database_name: &str,
    ) -> Result<PgConnection, StoreError> {
        let database_url = self.target_admin_database_url()?.to_owned();
        let mut url =
            Url::parse(&database_url).map_err(|error| StoreError::ReconciliationFailure {
                message: format!("invalid target admin database URL: {error}"),
            })?;
        url.set_path(&format!("/{database_name}"));
        Ok(PgConnection::connect(url.as_str()).await?)
    }

    async fn ensure_role(
        &self,
        role_name: &str,
        connection_limit: Option<i32>,
    ) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM pg_roles
             WHERE rolname = $1",
        )
        .bind(role_name)
        .fetch_one(&mut admin)
        .await?
            > 0;

        if !exists {
            let limit = connection_limit.unwrap_or(-1);
            sqlx::query(&format!(
                "CREATE ROLE {} LOGIN CONNECTION LIMIT {}",
                quote_identifier(role_name),
                limit
            ))
            .execute(&mut admin)
            .await?;
        }

        Ok(())
    }

    async fn set_role_connection_limit(
        &self,
        role_name: &str,
        connection_limit: Option<i32>,
    ) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        let limit = connection_limit.unwrap_or(-1);
        sqlx::query(&format!(
            "ALTER ROLE {} CONNECTION LIMIT {}",
            quote_identifier(role_name),
            limit
        ))
        .execute(&mut admin)
        .await?;
        Ok(())
    }

    async fn ensure_database(
        &self,
        database_name: &str,
        owner_role: &str,
    ) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM pg_database
             WHERE datname = $1",
        )
        .bind(database_name)
        .fetch_one(&mut admin)
        .await?
            > 0;

        if !exists {
            sqlx::query(&format!(
                "CREATE DATABASE {} OWNER {}",
                quote_identifier(database_name),
                quote_identifier(owner_role)
            ))
            .execute(&mut admin)
            .await?;
        }

        Ok(())
    }

    async fn set_database_owner(
        &self,
        database_name: &str,
        owner_role: &str,
    ) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        sqlx::query(&format!(
            "ALTER DATABASE {} OWNER TO {}",
            quote_identifier(database_name),
            quote_identifier(owner_role)
        ))
        .execute(&mut admin)
        .await?;
        Ok(())
    }

    async fn ensure_schema(
        &self,
        database_name: &str,
        schema_name: &str,
        owner_role: &str,
    ) -> Result<(), StoreError> {
        let mut target = self.target_database_connection(database_name).await?;
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM pg_namespace
             WHERE nspname = $1",
        )
        .bind(schema_name)
        .fetch_one(&mut target)
        .await?
            > 0;

        if !exists {
            sqlx::query(&format!(
                "CREATE SCHEMA {} AUTHORIZATION {}",
                quote_identifier(schema_name),
                quote_identifier(owner_role)
            ))
            .execute(&mut target)
            .await?;
        }

        Ok(())
    }

    async fn drop_schema(&self, database_name: &str, schema_name: &str) -> Result<(), StoreError> {
        let mut target = self.target_database_connection(database_name).await?;
        sqlx::query(&format!(
            "DROP SCHEMA IF EXISTS {} CASCADE",
            quote_identifier(schema_name)
        ))
        .execute(&mut target)
        .await?;
        Ok(())
    }

    async fn drop_database(&self, database_name: &str) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        sqlx::query(
            "SELECT pg_terminate_backend(pid)
             FROM pg_stat_activity
             WHERE datname = $1
               AND pid <> pg_backend_pid()",
        )
        .bind(database_name)
        .execute(&mut admin)
        .await?;
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS {}",
            quote_identifier(database_name)
        ))
        .execute(&mut admin)
        .await?;
        Ok(())
    }

    async fn drop_role(&self, role_name: &str) -> Result<(), StoreError> {
        let mut admin = self.admin_connection().await?;
        sqlx::query(&format!(
            "DROP ROLE IF EXISTS {}",
            quote_identifier(role_name)
        ))
        .execute(&mut admin)
        .await?;
        Ok(())
    }
}

fn capture_checkpoint(
    desired: &NormalizedCapsule,
    observed: &ObservedCapsuleState,
) -> ReconciliationCheckpoint {
    ReconciliationCheckpoint {
        database: DatabaseCheckpoint {
            name: desired.database.name.clone(),
            existed: observed.database.exists,
            owner_before: observed.database.owner_role.clone(),
        },
        roles: desired
            .roles
            .iter()
            .map(|role| {
                let observed_role = observed
                    .roles
                    .iter()
                    .find(|candidate| candidate.name == role.name);
                RoleCheckpoint {
                    name: role.name.clone(),
                    existed: observed_role
                        .map(|candidate| candidate.exists)
                        .unwrap_or(false),
                    connection_limit_before: observed_role
                        .and_then(|candidate| candidate.connection_limit),
                }
            })
            .collect(),
        schemas: desired
            .privilege_policy
            .managed_schemas
            .iter()
            .map(|schema_name| {
                let observed_schema = observed
                    .schemas
                    .iter()
                    .find(|candidate| candidate.name == *schema_name);
                SchemaCheckpoint {
                    name: schema_name.clone(),
                    existed: observed_schema
                        .map(|candidate| candidate.exists)
                        .unwrap_or(false),
                }
            })
            .collect(),
    }
}

fn normalize_connection_limit(value: i32) -> Option<i32> {
    if value == -1 { None } else { Some(value) }
}

fn drift_label(drift: &DriftReport) -> &'static str {
    match drift.overall.clone().unwrap_or(DriftSeverity::Safe) {
        DriftSeverity::Safe => "safe",
        DriftSeverity::Warning => "warning",
        DriftSeverity::Critical => "critical",
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn compute_event_hash(
    previous_hash: Option<&[u8]>,
    event_type: &str,
    payload: &serde_json::Value,
    operation_id: Option<Uuid>,
    run_id: Option<Uuid>,
) -> Result<Vec<u8>, StoreError> {
    let mut hasher = Sha256::new();
    if let Some(previous_hash) = previous_hash {
        hasher.update(previous_hash);
    }
    hasher.update(event_type.as_bytes());
    if let Some(operation_id) = operation_id {
        hasher.update(operation_id.as_bytes());
    }
    if let Some(run_id) = run_id {
        hasher.update(run_id.as_bytes());
    }
    hasher.update(serde_json::to_vec(payload)?);
    Ok(hasher.finalize().to_vec())
}

#[cfg(test)]
mod tests {
    use db_core::{
        capsule::{
            CapsuleSpec, DatabaseSpec, NormalizedCapsule, PrivilegePolicy, ProtectionClasses,
            ReconcileRequest, RoleSpec, SafetyPolicy,
        },
        operation::OperationState,
    };
    use uuid::Uuid;

    use crate::{StoreBootstrap, StoreRuntime};

    use super::ReconcileExecutionOptions;

    #[tokio::test]
    async fn observe_target_state_reports_missing_resources() {
        let Some(runtime) = runtime_from_env().await else {
            return;
        };
        let desired = sample_normalized("observe");

        let observed = runtime
            .observe_target_state(&desired)
            .await
            .expect("observation should succeed");

        assert!(!observed.database.exists);
        assert!(observed.roles.iter().all(|role| !role.exists));
    }

    #[tokio::test]
    async fn process_reconcile_operation_creates_database_and_role() {
        let Some(runtime) = runtime_from_env().await else {
            return;
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let app = format!("app{}", &suffix[..8]);
        let environment = String::from("prod");
        let spec = sample_spec(
            &app,
            &environment,
            vec![String::from("public"), String::from("analytics")],
        );
        let normalized = NormalizedCapsule::from_api(&app, &environment, &spec)
            .expect("capsule should normalize");
        let desired_fingerprint = db_core::fingerprint::desired_fingerprint(&normalized)
            .expect("desired fingerprint should build");
        let request_hash =
            db_core::fingerprint::request_fingerprint(&spec).expect("request hash should build");

        let operation = runtime
            .put_capsule_request(
                "test-principal",
                "idem-reconcile-success",
                &request_hash,
                &spec,
                &normalized,
                &desired_fingerprint,
            )
            .await
            .expect("capsule request should succeed");

        let reconcile = runtime
            .record_reconcile_request(
                &app,
                &environment,
                "test-principal",
                "idem-reconcile-run",
                &db_core::fingerprint::request_fingerprint(&ReconcileRequest::default())
                    .expect("reconcile request hash should build"),
                &ReconcileRequest::default(),
            )
            .await
            .expect("reconcile request should succeed");

        let completed = runtime
            .process_reconcile_operation(reconcile.operation_id)
            .await
            .expect("reconcile execution should succeed");

        assert_eq!(completed.state, OperationState::Succeeded.as_str());

        let observed = runtime
            .observe_target_state(&normalized)
            .await
            .expect("observation should succeed");

        assert!(observed.database.exists);
        assert!(
            observed
                .roles
                .iter()
                .any(|role| role.name == normalized.database.owner_role && role.exists)
        );
        assert!(
            observed
                .schemas
                .iter()
                .any(|schema| schema.name == "analytics" && schema.exists)
        );

        cleanup_target(&runtime, &normalized).await;
        let _ = operation;
    }

    #[tokio::test]
    async fn reconcile_rolls_back_on_failpoint() {
        let Some(runtime) = runtime_from_env().await else {
            return;
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let app = format!("app{}", &suffix[..8]);
        let environment = String::from("prod");
        let spec = sample_spec(
            &app,
            &environment,
            vec![String::from("public"), String::from("analytics")],
        );
        let normalized = NormalizedCapsule::from_api(&app, &environment, &spec)
            .expect("capsule should normalize");
        let desired_fingerprint = db_core::fingerprint::desired_fingerprint(&normalized)
            .expect("desired fingerprint should build");
        let request_hash =
            db_core::fingerprint::request_fingerprint(&spec).expect("request hash should build");

        runtime
            .put_capsule_request(
                "test-principal",
                "idem-reconcile-failpoint-put",
                &request_hash,
                &spec,
                &normalized,
                &desired_fingerprint,
            )
            .await
            .expect("capsule request should succeed");

        let reconcile = runtime
            .record_reconcile_request(
                &app,
                &environment,
                "test-principal",
                "idem-reconcile-failpoint-run",
                &db_core::fingerprint::request_fingerprint(&ReconcileRequest::default())
                    .expect("reconcile request hash should build"),
                &ReconcileRequest::default(),
            )
            .await
            .expect("reconcile request should succeed");

        let completed = runtime
            .process_reconcile_operation_with_options(
                reconcile.operation_id,
                ReconcileExecutionOptions {
                    fail_after_stage: Some(2),
                },
            )
            .await
            .expect("reconcile execution should complete with failure state");

        assert_eq!(completed.state, OperationState::Failed.as_str());

        let observed = runtime
            .observe_target_state(&normalized)
            .await
            .expect("observation should succeed");
        assert!(!observed.database.exists);
        assert!(observed.roles.iter().all(|role| !role.exists));
    }

    async fn runtime_from_env() -> Option<StoreRuntime> {
        let database_url = std::env::var("DATABASE_URL").ok()?;

        Some(
            StoreBootstrap {
                database_url: Some(database_url.clone()),
                target_admin_database_url: Some(database_url),
                run_migrations: true,
                max_connections: 4,
            }
            .bootstrap()
            .await
            .expect("store bootstrap should succeed"),
        )
    }

    fn sample_spec(app: &str, environment: &str, schemas: Vec<String>) -> CapsuleSpec {
        CapsuleSpec {
            database: DatabaseSpec {
                name: format!("app_{app}_{environment}"),
                owner_role: format!("{app}_{environment}_app"),
            },
            roles: vec![RoleSpec {
                name: format!("{app}_{environment}_app"),
                connection_limit: Some(50),
            }],
            privilege_policy: PrivilegePolicy {
                managed_schemas: schemas,
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

    fn sample_normalized(suffix: &str) -> NormalizedCapsule {
        let app = format!("observe{}", suffix);
        let environment = String::from("prod");
        NormalizedCapsule::from_api(
            &app,
            &environment,
            &sample_spec(&app, &environment, vec![String::from("public")]),
        )
        .expect("capsule should normalize")
    }

    async fn cleanup_target(runtime: &StoreRuntime, desired: &NormalizedCapsule) {
        let _ = runtime.drop_database(desired.database.name.as_str()).await;
        for role in &desired.roles {
            let _ = runtime.drop_role(role.name.as_str()).await;
        }
    }
}

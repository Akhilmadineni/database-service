BEGIN;

CREATE TABLE capsule (
    capsule_id uuid PRIMARY KEY,
    app text NOT NULL,
    environment text NOT NULL,
    status text NOT NULL,
    current_revision_id uuid NULL,
    current_desired_fingerprint text NULL,
    last_observed_fingerprint text NULL,
    last_plan_fingerprint text NULL,
    backup_class text NOT NULL,
    slo_class text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT capsule_app_environment_unique UNIQUE (app, environment)
);

CREATE TABLE capsule_revision (
    capsule_revision_id uuid PRIMARY KEY,
    capsule_id uuid NOT NULL REFERENCES capsule(capsule_id) ON DELETE CASCADE,
    revision_number bigint NOT NULL,
    spec_json jsonb NOT NULL,
    normalized_spec_json jsonb NOT NULL,
    desired_fingerprint text NOT NULL,
    created_by_principal text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT capsule_revision_capsule_revision_unique UNIQUE (capsule_id, revision_number),
    CONSTRAINT capsule_revision_capsule_fingerprint_unique UNIQUE (capsule_id, desired_fingerprint)
);

ALTER TABLE capsule
    ADD CONSTRAINT capsule_current_revision_fk
    FOREIGN KEY (current_revision_id)
    REFERENCES capsule_revision(capsule_revision_id)
    DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE operation (
    operation_id uuid PRIMARY KEY,
    capsule_id uuid NOT NULL REFERENCES capsule(capsule_id) ON DELETE CASCADE,
    operation_kind text NOT NULL,
    idempotency_key text NOT NULL,
    request_hash text NOT NULL,
    caller_principal text NOT NULL,
    state text NOT NULL,
    error_code text NULL,
    error_message text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    CONSTRAINT operation_idempotency_unique UNIQUE (caller_principal, capsule_id, operation_kind, idempotency_key)
);

CREATE TABLE reconciliation_run (
    reconciliation_run_id uuid PRIMARY KEY,
    operation_id uuid NOT NULL REFERENCES operation(operation_id) ON DELETE CASCADE,
    capsule_id uuid NOT NULL REFERENCES capsule(capsule_id) ON DELETE CASCADE,
    capsule_revision_id uuid NOT NULL REFERENCES capsule_revision(capsule_revision_id),
    prior_capsule_state text NOT NULL,
    final_capsule_state text NULL,
    drift_classification text NOT NULL,
    desired_fingerprint text NOT NULL,
    observed_fingerprint_before text NOT NULL,
    observed_fingerprint_after text NULL,
    plan_fingerprint text NULL,
    checkpoint_fingerprint text NULL,
    rollback_required boolean NOT NULL DEFAULT false,
    rollback_outcome text NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL
);

CREATE TABLE checkpoint (
    checkpoint_id uuid PRIMARY KEY,
    reconciliation_run_id uuid NOT NULL UNIQUE REFERENCES reconciliation_run(reconciliation_run_id) ON DELETE CASCADE,
    checkpoint_json jsonb NOT NULL,
    checkpoint_fingerprint text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_event (
    audit_event_id uuid PRIMARY KEY,
    capsule_id uuid NULL REFERENCES capsule(capsule_id),
    operation_id uuid NULL REFERENCES operation(operation_id),
    reconciliation_run_id uuid NULL REFERENCES reconciliation_run(reconciliation_run_id),
    event_type text NOT NULL,
    payload_json jsonb NOT NULL,
    prev_hash bytea NULL,
    event_hash bytea NOT NULL,
    occurred_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX operation_capsule_created_at_idx
    ON operation (capsule_id, created_at DESC);

CREATE INDEX reconciliation_run_capsule_started_at_idx
    ON reconciliation_run (capsule_id, started_at DESC);

CREATE INDEX audit_event_capsule_occurred_at_idx
    ON audit_event (capsule_id, occurred_at DESC);

COMMIT;

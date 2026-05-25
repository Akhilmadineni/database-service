use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CapsuleSpec {
    pub database: DatabaseSpec,
    pub roles: Vec<RoleSpec>,
    pub privilege_policy: PrivilegePolicy,
    pub protection_classes: ProtectionClasses,
    pub safety_policy: SafetyPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseSpec {
    pub name: String,
    pub owner_role: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleSpec {
    pub name: String,
    pub connection_limit: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrivilegePolicy {
    pub managed_schemas: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtectionClasses {
    pub backup_class: String,
    pub slo_class: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SafetyPolicy {
    pub allow_destructive_mutation: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReconcileRequest {
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CapsuleView {
    pub capsule_id: Uuid,
    pub app: String,
    pub environment: String,
    pub status: String,
    pub current_revision_number: Option<i64>,
    pub current_desired_fingerprint: Option<String>,
    pub backup_class: String,
    pub slo_class: String,
}

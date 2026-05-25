use serde::{Deserialize, Serialize};

use crate::{
    capsule::NormalizedCapsule,
    observed::{ObservedCapsuleState, ObservedRoleState},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    Safe,
    Warning,
    Critical,
}

impl DriftSeverity {
    pub fn worst(left: Self, right: Self) -> Self {
        use DriftSeverity::{Critical, Safe, Warning};

        match (left, right) {
            (Critical, _) | (_, Critical) => Critical,
            (Warning, _) | (_, Warning) => Warning,
            _ => Safe,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftKind {
    MissingDatabase,
    DatabaseOwnerMismatch,
    MissingRole,
    RoleConnectionLimitMismatch,
    MissingSchema,
    ActiveSessionsDuringMutation,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DriftItem {
    pub kind: DriftKind,
    pub severity: DriftSeverity,
    pub score: u32,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct DriftReport {
    pub items: Vec<DriftItem>,
    pub overall: Option<DriftSeverity>,
    pub requires_reconcile: bool,
}

pub fn classify_drift(desired: &NormalizedCapsule, observed: &ObservedCapsuleState) -> DriftReport {
    let mut items = Vec::new();

    if !observed.database.exists {
        items.push(DriftItem {
            kind: DriftKind::MissingDatabase,
            severity: DriftSeverity::Safe,
            score: 30,
            message: format!("database `{}` is missing", desired.database.name),
        });
    } else if observed.database.owner_role.as_deref() != Some(desired.database.owner_role.as_str())
    {
        items.push(DriftItem {
            kind: DriftKind::DatabaseOwnerMismatch,
            severity: if observed.runtime.active_session_count > 0 {
                DriftSeverity::Warning
            } else {
                DriftSeverity::Safe
            },
            score: if observed.runtime.active_session_count > 0 {
                70
            } else {
                40
            },
            message: format!(
                "database owner `{}` does not match desired owner `{}`",
                observed.database.owner_role.as_deref().unwrap_or("<none>"),
                desired.database.owner_role
            ),
        });
    }

    for desired_role in &desired.roles {
        let observed_role = observed
            .roles
            .iter()
            .find(|role| role.name == desired_role.name)
            .cloned()
            .unwrap_or_else(|| ObservedRoleState {
                name: desired_role.name.clone(),
                exists: false,
                connection_limit: None,
            });

        if !observed_role.exists {
            items.push(DriftItem {
                kind: DriftKind::MissingRole,
                severity: DriftSeverity::Safe,
                score: 20,
                message: format!("role `{}` is missing", desired_role.name),
            });
        } else if observed_role.connection_limit != desired_role.connection_limit {
            items.push(DriftItem {
                kind: DriftKind::RoleConnectionLimitMismatch,
                severity: DriftSeverity::Safe,
                score: 10,
                message: format!(
                    "role `{}` connection limit differs (observed {:?}, desired {:?})",
                    desired_role.name,
                    observed_role.connection_limit,
                    desired_role.connection_limit
                ),
            });
        }
    }

    for schema in &desired.privilege_policy.managed_schemas {
        let exists = observed
            .schemas
            .iter()
            .find(|candidate| candidate.name == *schema)
            .map(|candidate| candidate.exists)
            .unwrap_or(false);

        let should_emit = if observed.database.exists {
            !exists
        } else {
            schema != "public"
        };

        if should_emit {
            items.push(DriftItem {
                kind: DriftKind::MissingSchema,
                severity: if schema == "public" {
                    DriftSeverity::Critical
                } else {
                    DriftSeverity::Safe
                },
                score: if schema == "public" { 90 } else { 15 },
                message: format!("managed schema `{schema}` is missing"),
            });
        }
    }

    if observed.runtime.active_session_count > 0 && !items.is_empty() {
        items.push(DriftItem {
            kind: DriftKind::ActiveSessionsDuringMutation,
            severity: DriftSeverity::Warning,
            score: 80,
            message: format!(
                "{} active sessions are present while reconciliation would mutate state",
                observed.runtime.active_session_count
            ),
        });
    }

    let overall = items.iter().fold(None, |current, item| match current {
        Some(existing) => Some(DriftSeverity::worst(existing, item.severity.clone())),
        None => Some(item.severity.clone()),
    });

    DriftReport {
        requires_reconcile: !items.is_empty(),
        items,
        overall,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        capsule::{
            CapsuleSpec, DatabaseSpec, NormalizedCapsule, PrivilegePolicy, ProtectionClasses,
            RoleSpec, SafetyPolicy,
        },
        observed::{
            ObservedCapsuleState, ObservedDatabaseState, ObservedRoleState, ObservedRuntimeState,
            ObservedSchemaState,
        },
    };

    use super::{DriftKind, DriftSeverity, classify_drift};

    #[test]
    fn classifies_owner_mismatch_with_sessions_as_warning() {
        let desired = NormalizedCapsule::from_api(
            "clustr",
            "prod",
            &CapsuleSpec {
                database: DatabaseSpec {
                    name: String::from("app_clustr_prod"),
                    owner_role: String::from("clustr_prod_app"),
                },
                roles: vec![RoleSpec {
                    name: String::from("clustr_prod_app"),
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
            },
        )
        .expect("desired capsule should normalize");

        let observed = ObservedCapsuleState {
            app: String::from("clustr"),
            environment: String::from("prod"),
            database: ObservedDatabaseState {
                name: String::from("app_clustr_prod"),
                exists: true,
                owner_role: Some(String::from("other_owner")),
            },
            roles: vec![ObservedRoleState {
                name: String::from("clustr_prod_app"),
                exists: true,
                connection_limit: Some(50),
            }],
            schemas: vec![ObservedSchemaState {
                name: String::from("public"),
                exists: true,
            }],
            runtime: ObservedRuntimeState {
                active_session_count: 3,
                active_lock_count: 0,
            },
        };

        let report = classify_drift(&desired, &observed);
        assert!(report.requires_reconcile);
        assert_eq!(report.overall, Some(DriftSeverity::Warning));
        assert!(
            report
                .items
                .iter()
                .any(|item| item.kind == DriftKind::DatabaseOwnerMismatch)
        );
        assert!(
            report
                .items
                .iter()
                .any(|item| item.kind == DriftKind::ActiveSessionsDuringMutation)
        );
    }
}

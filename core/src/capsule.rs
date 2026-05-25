use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CapsuleValidationError;

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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProtectionClasses {
    pub backup_class: String,
    pub slo_class: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedCapsule {
    pub identity: CapsuleIdentity,
    pub database: NormalizedDatabase,
    pub roles: Vec<NormalizedRole>,
    pub privilege_policy: NormalizedPrivilegePolicy,
    pub protection_classes: ProtectionClasses,
    pub safety_policy: SafetyPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapsuleIdentity {
    pub app: String,
    pub environment: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedDatabase {
    pub name: String,
    pub owner_role: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalizedRole {
    pub name: String,
    pub connection_limit: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedPrivilegePolicy {
    pub managed_schemas: Vec<String>,
}

impl NormalizedCapsule {
    pub fn from_api(
        app: &str,
        environment: &str,
        spec: &CapsuleSpec,
    ) -> Result<Self, CapsuleValidationError> {
        let app = normalize_segment(app, "app")?;
        let environment = normalize_segment(environment, "environment")?;
        let database_name = normalize_pg_identifier(&spec.database.name, "database.name")?;
        let owner_role = normalize_pg_identifier(&spec.database.owner_role, "database.owner_role")?;

        let mut roles = spec
            .roles
            .iter()
            .map(|role| {
                let name = normalize_pg_identifier(&role.name, "roles[].name")?;
                validate_connection_limit(role.connection_limit)?;
                Ok(NormalizedRole {
                    name,
                    connection_limit: role.connection_limit,
                })
            })
            .collect::<Result<Vec<_>, CapsuleValidationError>>()?;

        if roles.is_empty() {
            return Err(CapsuleValidationError::MissingRoles);
        }

        roles.sort();
        roles.dedup_by(|left, right| left.name == right.name);

        let mut managed_schemas = spec
            .privilege_policy
            .managed_schemas
            .iter()
            .map(|schema| normalize_pg_identifier(schema, "privilege_policy.managed_schemas[]"))
            .collect::<Result<Vec<_>, CapsuleValidationError>>()?;

        if managed_schemas.is_empty() {
            return Err(CapsuleValidationError::MissingManagedSchemas);
        }

        managed_schemas.sort();
        managed_schemas.dedup();

        let backup_class = normalize_class_name(
            &spec.protection_classes.backup_class,
            "protection_classes.backup_class",
        )?;
        let slo_class = normalize_class_name(
            &spec.protection_classes.slo_class,
            "protection_classes.slo_class",
        )?;

        let capsule = Self {
            identity: CapsuleIdentity { app, environment },
            database: NormalizedDatabase {
                name: database_name,
                owner_role,
            },
            roles,
            privilege_policy: NormalizedPrivilegePolicy { managed_schemas },
            protection_classes: ProtectionClasses {
                backup_class,
                slo_class,
            },
            safety_policy: SafetyPolicy {
                allow_destructive_mutation: spec.safety_policy.allow_destructive_mutation,
            },
        };

        capsule.validate_cross_references()?;
        Ok(capsule)
    }

    fn validate_cross_references(&self) -> Result<(), CapsuleValidationError> {
        if !self
            .roles
            .iter()
            .any(|role| role.name == self.database.owner_role)
        {
            return Err(CapsuleValidationError::OwnerRoleMissing {
                owner_role: self.database.owner_role.clone(),
            });
        }

        Ok(())
    }
}

fn normalize_segment(value: &str, field: &'static str) -> Result<String, CapsuleValidationError> {
    let normalized = value.trim().to_ascii_lowercase();
    let is_valid = (2..=50).contains(&normalized.len())
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');

    if is_valid {
        Ok(normalized)
    } else {
        Err(CapsuleValidationError::InvalidField {
            field,
            message: String::from("use lowercase letters, numbers, and hyphens only"),
        })
    }
}

fn normalize_pg_identifier(
    value: &str,
    field: &'static str,
) -> Result<String, CapsuleValidationError> {
    let normalized = value.trim().to_ascii_lowercase();
    let is_valid = !normalized.is_empty()
        && normalized.len() <= 63
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');

    if is_valid {
        Ok(normalized)
    } else {
        Err(CapsuleValidationError::InvalidField {
            field,
            message: String::from(
                "use lowercase letters, numbers, and underscores only, max 63 chars",
            ),
        })
    }
}

fn normalize_class_name(
    value: &str,
    field: &'static str,
) -> Result<String, CapsuleValidationError> {
    let normalized = value.trim().to_ascii_lowercase();
    let is_valid = !normalized.is_empty()
        && normalized.len() <= 32
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');

    if is_valid {
        Ok(normalized)
    } else {
        Err(CapsuleValidationError::InvalidField {
            field,
            message: String::from("use lowercase letters, numbers, hyphens, and underscores only"),
        })
    }
}

fn validate_connection_limit(value: Option<i32>) -> Result<(), CapsuleValidationError> {
    if let Some(limit) = value
        && limit < -1
    {
        return Err(CapsuleValidationError::InvalidField {
            field: "roles[].connection_limit",
            message: String::from("must be greater than or equal to -1"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CapsuleSpec, DatabaseSpec, NormalizedCapsule, PrivilegePolicy, ProtectionClasses, RoleSpec,
        SafetyPolicy,
    };

    #[test]
    fn normalizes_roles_and_schemas_deterministically() {
        let spec = CapsuleSpec {
            database: DatabaseSpec {
                name: String::from("app_clustr_prod"),
                owner_role: String::from("clustr_prod_app"),
            },
            roles: vec![
                RoleSpec {
                    name: String::from("clustr_prod_reader"),
                    connection_limit: Some(10),
                },
                RoleSpec {
                    name: String::from("clustr_prod_app"),
                    connection_limit: Some(50),
                },
            ],
            privilege_policy: PrivilegePolicy {
                managed_schemas: vec![String::from("analytics"), String::from("public")],
            },
            protection_classes: ProtectionClasses {
                backup_class: String::from("Standard"),
                slo_class: String::from("Gold"),
            },
            safety_policy: SafetyPolicy {
                allow_destructive_mutation: false,
            },
        };

        let normalized = NormalizedCapsule::from_api("clustr", "prod", &spec)
            .expect("normalization should succeed");

        assert_eq!(normalized.roles[0].name, "clustr_prod_app");
        assert_eq!(normalized.roles[1].name, "clustr_prod_reader");
        assert_eq!(
            normalized.privilege_policy.managed_schemas,
            vec!["analytics", "public"]
        );
        assert_eq!(normalized.protection_classes.backup_class, "standard");
        assert_eq!(normalized.protection_classes.slo_class, "gold");
    }

    #[test]
    fn rejects_missing_owner_role_reference() {
        let spec = CapsuleSpec {
            database: DatabaseSpec {
                name: String::from("app_clustr_prod"),
                owner_role: String::from("missing_owner"),
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
        };

        let error = NormalizedCapsule::from_api("clustr", "prod", &spec)
            .expect_err("missing owner role should fail");
        assert!(error.to_string().contains("owner role"));
    }
}

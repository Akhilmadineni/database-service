use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{capsule::NormalizedCapsule, observed::ObservedCapsuleState, plan::ReconciliationPlan};

#[derive(Clone, Copy, Debug)]
pub enum FingerprintDomain {
    Desired,
    Observed,
    Plan,
    Checkpoint,
    Request,
}

impl FingerprintDomain {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Desired => "acr:desired:v1",
            Self::Observed => "acr:observed:v1",
            Self::Plan => "acr:plan:v1",
            Self::Checkpoint => "acr:checkpoint:v1",
            Self::Request => "acr:request:v1",
        }
    }
}

pub fn desired_fingerprint(capsule: &NormalizedCapsule) -> Result<String, serde_json::Error> {
    fingerprint_value(FingerprintDomain::Desired, capsule)
}

pub fn observed_fingerprint(observed: &ObservedCapsuleState) -> Result<String, serde_json::Error> {
    fingerprint_value(FingerprintDomain::Observed, observed)
}

pub fn plan_fingerprint(plan: &ReconciliationPlan) -> Result<String, serde_json::Error> {
    fingerprint_value(FingerprintDomain::Plan, plan)
}

pub fn checkpoint_fingerprint<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    fingerprint_value(FingerprintDomain::Checkpoint, value)
}

pub fn request_fingerprint<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    fingerprint_value(FingerprintDomain::Request, value)
}

pub fn fingerprint_value<T: Serialize>(
    domain: FingerprintDomain,
    value: &T,
) -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_value(value)?;
    canonicalize_json_value(&mut json);
    let mut hasher = Sha256::new();
    hasher.update(domain.prefix().as_bytes());
    hasher.update([0]);
    hasher.update(serde_json::to_vec(&json)?);
    Ok(hex::encode(hasher.finalize()))
}

pub fn canonicalize_json_value(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for item in values.iter_mut() {
                canonicalize_json_value(item);
            }
            values.sort_by_key(|left| left.to_string());
        }
        Value::Object(map) => {
            let mut entries = map
                .iter_mut()
                .map(|(key, value)| {
                    canonicalize_json_value(value);
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

#[cfg(test)]
mod tests {
    use crate::capsule::{
        CapsuleSpec, DatabaseSpec, NormalizedCapsule, PrivilegePolicy, ProtectionClasses, RoleSpec,
        SafetyPolicy,
    };

    use super::desired_fingerprint;

    #[test]
    fn equivalent_capsules_share_fingerprint() {
        let left = CapsuleSpec {
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
                managed_schemas: vec![String::from("public"), String::from("analytics")],
            },
            protection_classes: ProtectionClasses {
                backup_class: String::from("standard"),
                slo_class: String::from("gold"),
            },
            safety_policy: SafetyPolicy {
                allow_destructive_mutation: false,
            },
        };

        let right = CapsuleSpec {
            database: DatabaseSpec {
                name: String::from("app_clustr_prod"),
                owner_role: String::from("clustr_prod_app"),
            },
            roles: vec![
                RoleSpec {
                    name: String::from("clustr_prod_app"),
                    connection_limit: Some(50),
                },
                RoleSpec {
                    name: String::from("clustr_prod_reader"),
                    connection_limit: Some(10),
                },
            ],
            privilege_policy: PrivilegePolicy {
                managed_schemas: vec![String::from("analytics"), String::from("public")],
            },
            protection_classes: ProtectionClasses {
                backup_class: String::from("standard"),
                slo_class: String::from("gold"),
            },
            safety_policy: SafetyPolicy {
                allow_destructive_mutation: false,
            },
        };

        let left = NormalizedCapsule::from_api("clustr", "prod", &left)
            .expect("left capsule should normalize");
        let right = NormalizedCapsule::from_api("clustr", "prod", &right)
            .expect("right capsule should normalize");

        assert_eq!(
            desired_fingerprint(&left).expect("left fingerprint should build"),
            desired_fingerprint(&right).expect("right fingerprint should build")
        );
    }
}

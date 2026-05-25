use serde::{Deserialize, Serialize};

use crate::drift::{DriftKind, DriftReport, DriftSeverity};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageKind {
    EnsureRole,
    SetRoleConnectionLimit,
    EnsureDatabase,
    SetDatabaseOwner,
    EnsureSchema,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PlanStage {
    pub kind: StageKind,
    pub target: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReconciliationPlan {
    pub stages: Vec<PlanStage>,
    pub blocks_autonomous_apply: bool,
}

pub fn plan_from_drift(drift: &DriftReport) -> ReconciliationPlan {
    let mut stages = Vec::new();
    let mut blocks_autonomous_apply = false;

    for item in &drift.items {
        if item.severity == DriftSeverity::Critical {
            blocks_autonomous_apply = true;
        }

        match item.kind {
            DriftKind::MissingRole => stages.push(PlanStage {
                kind: StageKind::EnsureRole,
                target: item.message.clone(),
            }),
            DriftKind::RoleConnectionLimitMismatch => stages.push(PlanStage {
                kind: StageKind::SetRoleConnectionLimit,
                target: item.message.clone(),
            }),
            DriftKind::MissingDatabase => stages.push(PlanStage {
                kind: StageKind::EnsureDatabase,
                target: item.message.clone(),
            }),
            DriftKind::DatabaseOwnerMismatch => stages.push(PlanStage {
                kind: StageKind::SetDatabaseOwner,
                target: item.message.clone(),
            }),
            DriftKind::MissingSchema => stages.push(PlanStage {
                kind: StageKind::EnsureSchema,
                target: item.message.clone(),
            }),
            DriftKind::ActiveSessionsDuringMutation => {
                blocks_autonomous_apply = true;
            }
        }
    }

    stages.sort_by_key(|stage| (stage_order(&stage.kind), stage.target.clone()));

    ReconciliationPlan {
        stages,
        blocks_autonomous_apply,
    }
}

fn stage_order(kind: &StageKind) -> u8 {
    match kind {
        StageKind::EnsureRole => 0,
        StageKind::SetRoleConnectionLimit => 1,
        StageKind::EnsureDatabase => 2,
        StageKind::SetDatabaseOwner => 3,
        StageKind::EnsureSchema => 4,
    }
}

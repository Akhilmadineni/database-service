use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservedCapsuleState {
    pub app: String,
    pub environment: String,
    pub database: ObservedDatabaseState,
    pub roles: Vec<ObservedRoleState>,
    pub schemas: Vec<ObservedSchemaState>,
    pub runtime: ObservedRuntimeState,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservedDatabaseState {
    pub name: String,
    pub exists: bool,
    pub owner_role: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedRoleState {
    pub name: String,
    pub exists: bool,
    pub connection_limit: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedSchemaState {
    pub name: String,
    pub exists: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservedRuntimeState {
    pub active_session_count: u32,
    pub active_lock_count: u32,
}

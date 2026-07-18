use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct TaskTracker {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub instance_url: String,
}

#[derive(Serialize)]
pub struct CreateTaskTrackerRequest {
    pub name: String,
    pub instance_url: String,
    pub api_key: String,
}

#[derive(Default, Serialize)]
pub struct UpdateTaskTrackerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

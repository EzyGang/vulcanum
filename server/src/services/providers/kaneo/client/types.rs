use crate::util::serde::deserialize_null_default;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaneoWorkspace {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KaneoTask {
    pub id: String,
    pub project_id: String,
    pub number: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub assignee_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub labels: Vec<KaneoTaskLabel>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KaneoTaskLabel {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KaneoBoardColumn {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    pub is_final: Option<bool>,
    pub tasks: Vec<KaneoTask>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KaneoBoardData {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub columns: Vec<KaneoBoardColumn>,
    #[serde(default)]
    pub planned_tasks: Vec<KaneoTask>,
    #[serde(default)]
    pub archived_tasks: Vec<KaneoTask>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KaneoBoardResponse {
    pub data: KaneoBoardData,
}

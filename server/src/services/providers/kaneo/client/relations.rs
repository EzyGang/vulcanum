use serde::{Deserialize, Serialize};

use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KaneoTaskRelation {
    source_task_id: String,
    target_task_id: String,
    relation_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskRelationBody<'a> {
    source_task_id: &'a str,
    target_task_id: &'a str,
    relation_type: &'static str,
}

impl KaneoClient {
    pub(crate) async fn ensure_task_blocks(
        &self,
        source_task_id: &str,
        target_task_id: &str,
    ) -> Result<(), KaneoError> {
        let client = self.build_client()?;
        let list_path = format!("/task-relation/{source_task_id}");
        let start = std::time::Instant::now();
        let relations = client
            .get::<Vec<KaneoTaskRelation>>(&list_path)
            .await
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;
        log_kaneo_result("GET", &list_path, duration_ms, &relations);

        if relations?.iter().any(|relation| {
            relation.relation_type == "blocks"
                && ((relation.source_task_id == source_task_id
                    && relation.target_task_id == target_task_id)
                    || (relation.source_task_id == target_task_id
                        && relation.target_task_id == source_task_id))
        }) {
            return Ok(());
        }

        let path = "/task-relation";
        let body = CreateTaskRelationBody {
            source_task_id,
            target_task_id,
            relation_type: "blocks",
        };
        let start = std::time::Instant::now();
        let result = client
            .post::<_, KaneoTaskRelation>(path, &body)
            .await
            .map(|_| ())
            .map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;
        log_kaneo_result("POST", path, duration_ms, &result);
        result
    }
}

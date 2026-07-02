use std::collections::HashMap;

use vulcanum_shared::api_types::{AgentConfigPayload, JobResponse};

pub(crate) fn job_runtime_secrets(job: &JobResponse) -> HashMap<String, String> {
    let mut secrets = HashMap::new();
    secrets.insert(
        "PROVIDER_INSTANCE_URL".to_owned(),
        job.provider_instance_url.clone(),
    );
    secrets.insert("PROVIDER_API_KEY".to_owned(), job.provider_api_key.clone());
    secrets.insert(
        "EXTERNAL_PROJECT_ID".to_owned(),
        job.external_project_id.clone(),
    );
    secrets.insert(
        "EXTERNAL_WORKSPACE_ID".to_owned(),
        job.external_workspace_id.clone(),
    );
    secrets.insert("EXTERNAL_TASK_ID".to_owned(), job.external_task_ref.clone());
    if let Some(token) = &job.github_token {
        secrets.insert("GITHUB_TOKEN".to_owned(), token.clone());
    }
    for (key, value) in &job.model_provider_env {
        secrets.insert(key.clone(), value.clone());
    }
    if let AgentConfigPayload::OpenCode {
        auth_content: Some(auth_content),
        ..
    } = &job.agent_config
    {
        secrets.insert("OPENCODE_AUTH_CONTENT".to_owned(), auth_content.clone());
    }
    secrets
}

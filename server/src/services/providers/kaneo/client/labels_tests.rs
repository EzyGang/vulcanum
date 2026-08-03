use crate::services::providers::kaneo::client::labels::{
    create_label_body, task_label_ids_by_name,
};
use crate::services::providers::kaneo::client::types::{KaneoTask, KaneoTaskLabel};

#[test]
fn task_label_creation_serializes_attachment_owner() {
    let body = create_label_body("workspace-1", "Review needed", "#D97706", Some("task-1"));
    let value = serde_json::to_value(body);
    let expected = serde_json::json!({
        "name": "Review needed",
        "color": "#D97706",
        "workspaceId": "workspace-1",
        "taskId": "task-1",
    });

    assert_eq!(value.as_ref().ok(), Some(&expected));
}

#[test]
fn task_label_removal_resolves_attached_copy_by_name() {
    let task = KaneoTask {
        id: "task-1".to_owned(),
        project_id: "project-1".to_owned(),
        number: None,
        position: None,
        title: "Task".to_owned(),
        description: None,
        status: "in-progress".to_owned(),
        priority: "low".to_owned(),
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        due_date: None,
        start_date: None,
        user_id: None,
        updated_at: None,
        assignee_name: None,
        assignee_id: None,
        labels: vec![
            KaneoTaskLabel {
                id: "attached-copy".to_owned(),
                name: "Review needed".to_owned(),
                color: "#D97706".to_owned(),
            },
            KaneoTaskLabel {
                id: "unrelated".to_owned(),
                name: "Bug".to_owned(),
                color: "#DC2626".to_owned(),
            },
        ],
    };

    assert_eq!(
        task_label_ids_by_name(&task, "Review needed"),
        vec!["attached-copy"]
    );
}

use crate::models::providers::model::IntegrationLabel;
use crate::services::work_runs::service::lifecycle_labels::{
    lifecycle_label_template, LifecycleLabelState,
};

#[test]
fn lifecycle_template_ignores_copy_attached_to_another_task() {
    let labels = vec![
        IntegrationLabel {
            id: "attached-copy".to_owned(),
            name: "Review needed".to_owned(),
            color: "#D97706".to_owned(),
            task_id: Some("task-1".to_owned()),
        },
        IntegrationLabel {
            id: "workspace-template".to_owned(),
            name: "Review needed".to_owned(),
            color: "#D97706".to_owned(),
            task_id: None,
        },
    ];

    let template = lifecycle_label_template(&labels, LifecycleLabelState::ReviewNeeded);

    assert_eq!(
        template.map(|label| label.id.as_str()),
        Some("workspace-template")
    );
}

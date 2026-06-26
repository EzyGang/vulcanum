use crate::models::providers::model::IntegrationColumn;
use crate::services::task_board::service::default_column_status;

fn column(slug: &str, is_final: Option<bool>) -> IntegrationColumn {
    IntegrationColumn {
        id: slug.to_owned(),
        name: slug.to_owned(),
        slug: slug.to_owned(),
        is_final,
    }
}

#[test]
fn default_column_status_prefers_first_non_final_column() {
    let columns = vec![
        column("done", Some(true)),
        column("in-progress", Some(false)),
    ];

    assert_eq!(default_column_status(&columns), "in-progress");
}

#[test]
fn default_column_status_falls_back_when_provider_has_no_columns() {
    assert_eq!(default_column_status(&[]), "planned");
}

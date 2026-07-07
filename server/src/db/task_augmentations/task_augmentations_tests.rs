use uuid::Uuid;

use crate::db::task_augmentations::{IncrementTaskUsageParams, TaskAugmentationsRepository};
use crate::test_helpers;

fn usage_params<'a>(
    team_id: Uuid,
    project_config_id: Uuid,
    external_task_ref: &'a str,
    tokens: [i64; 5],
) -> IncrementTaskUsageParams<'a> {
    let [tokens_used, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens] = tokens;
    IncrementTaskUsageParams {
        team_id,
        project_config_id,
        external_task_ref,
        tokens_used,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
    }
}

#[sqlx::test]
async fn increment_usage_accumulates_counts_and_tokens_per_task(pool: sqlx::PgPool) {
    let repo = TaskAugmentationsRepository::new();
    let project_config_id =
        test_helpers::insert_project_config(&pool, "task-augment-accumulate").await;

    repo.increment_usage(
        &pool,
        usage_params(
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            "task-shared",
            [10, 4, 6, 1, 2],
        ),
    )
    .await
    .expect("Should record first finished run usage");
    repo.increment_usage(
        &pool,
        usage_params(
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            "task-shared",
            [30, 11, 19, 3, 5],
        ),
    )
    .await
    .expect("Should accumulate second finished run usage");

    let task_refs = vec!["task-shared".to_owned()];
    let rows = repo
        .list_for_task_refs(
            &pool,
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            &task_refs,
        )
        .await
        .expect("Should load task augmentation");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].external_task_ref, "task-shared");
    assert_eq!(rows[0].tokens_used, 40);
    assert_eq!(rows[0].input_tokens, 15);
    assert_eq!(rows[0].output_tokens, 25);
    assert_eq!(rows[0].cache_read_tokens, 4);
    assert_eq!(rows[0].cache_write_tokens, 7);
    assert_eq!(rows[0].finished_runs_count, 2);
}

#[sqlx::test]
async fn list_for_task_refs_returns_visible_augmentations_in_board_order(pool: sqlx::PgPool) {
    let repo = TaskAugmentationsRepository::new();
    let project_config_id = test_helpers::insert_project_config(&pool, "task-augment-board").await;
    let other_project_config_id =
        test_helpers::insert_project_config(&pool, "task-augment-other-project").await;
    let other_team_id = test_helpers::insert_team(&pool, "Task augmentation other team").await;
    let other_team_project_config_id = test_helpers::insert_project_config_for_team(
        &pool,
        other_team_id,
        "task-augment-other-team",
    )
    .await;

    repo.increment_usage(
        &pool,
        usage_params(
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            "task-a",
            [10, 1, 2, 3, 4],
        ),
    )
    .await
    .expect("Should seed task-a usage");
    repo.increment_usage(
        &pool,
        usage_params(
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            "task-b",
            [20, 5, 6, 7, 8],
        ),
    )
    .await
    .expect("Should seed task-b usage");
    repo.increment_usage(
        &pool,
        usage_params(
            test_helpers::DEFAULT_TEAM_ID,
            other_project_config_id,
            "task-c",
            [99, 99, 99, 99, 99],
        ),
    )
    .await
    .expect("Should seed same-team other-project usage");
    repo.increment_usage(
        &pool,
        usage_params(
            other_team_id,
            other_team_project_config_id,
            "task-c",
            [88, 88, 88, 88, 88],
        ),
    )
    .await
    .expect("Should seed other-team usage");

    let task_refs = vec![
        "task-c".to_owned(),
        "task-b".to_owned(),
        "task-missing".to_owned(),
        "task-a".to_owned(),
    ];
    let rows = repo
        .list_for_task_refs(
            &pool,
            test_helpers::DEFAULT_TEAM_ID,
            project_config_id,
            &task_refs,
        )
        .await
        .expect("Should load visible task augmentations");

    let returned_refs = rows
        .iter()
        .map(|row| row.external_task_ref.as_str())
        .collect::<Vec<_>>();

    assert_eq!(returned_refs, vec!["task-b", "task-a"]);
    assert_eq!(rows[0].tokens_used, 20);
    assert_eq!(rows[0].input_tokens, 5);
    assert_eq!(rows[0].output_tokens, 6);
    assert_eq!(rows[0].cache_read_tokens, 7);
    assert_eq!(rows[0].cache_write_tokens, 8);
    assert_eq!(rows[0].finished_runs_count, 1);
    assert_eq!(rows[1].tokens_used, 10);
    assert_eq!(rows[1].input_tokens, 1);
    assert_eq!(rows[1].output_tokens, 2);
    assert_eq!(rows[1].cache_read_tokens, 3);
    assert_eq!(rows[1].cache_write_tokens, 4);
    assert_eq!(rows[1].finished_runs_count, 1);
}

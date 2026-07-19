use crate::db::project_usage::{IncrementProjectUsageParams, ProjectUsageRepository};
use crate::test_helpers;

fn usage_params(project_config_id: uuid::Uuid, tokens: [i64; 5]) -> IncrementProjectUsageParams {
    let [tokens_used, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens] = tokens;
    IncrementProjectUsageParams {
        project_config_id,
        tokens_used,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
    }
}

#[sqlx::test]
async fn daily_usage_accumulates_and_returns_project_summary(pool: sqlx::PgPool) {
    let project_config_id = test_helpers::insert_project_config(&pool, "project-usage").await;
    let repo = ProjectUsageRepository::new();

    repo.increment_daily(&pool, usage_params(project_config_id, [100, 40, 60, 7, 3]))
        .await
        .expect("record first project usage");
    repo.increment_daily(&pool, usage_params(project_config_id, [25, 10, 15, 2, 1]))
        .await
        .expect("record second project usage");

    let summary = repo
        .summary(&pool, project_config_id)
        .await
        .expect("load project usage summary");

    assert_eq!(summary.total.tokens_used, 125);
    assert_eq!(summary.total.input_tokens, 50);
    assert_eq!(summary.total.output_tokens, 75);
    assert_eq!(summary.total.cache_read_tokens, 9);
    assert_eq!(summary.total.cache_write_tokens, 4);
    assert_eq!(summary.total.finished_runs_count, 2);
    assert_eq!(summary.this_week.tokens_used, 125);
    assert_eq!(summary.this_week.finished_runs_count, 2);
}

#[sqlx::test]
async fn weekly_usage_excludes_days_before_monday(pool: sqlx::PgPool) {
    let project_config_id = test_helpers::insert_project_config(&pool, "project-usage-week").await;
    let repo = ProjectUsageRepository::new();

    sqlx::query(
        r#"INSERT INTO project_usage_daily (
            project_config_id, usage_date, tokens_used, input_tokens, output_tokens,
            cache_read_tokens, cache_write_tokens, finished_runs_count
        )
        VALUES (
            $1,
            DATE_TRUNC('week', statement_timestamp() AT TIME ZONE 'UTC')::DATE - 1,
            90, 30, 60, 0, 0, 1
        )"#,
    )
    .bind(project_config_id)
    .execute(&pool)
    .await
    .expect("seed prior-week usage");
    repo.increment_daily(&pool, usage_params(project_config_id, [10, 4, 6, 1, 0]))
        .await
        .expect("record current-week usage");

    let summary = repo
        .summary(&pool, project_config_id)
        .await
        .expect("load weekly project usage");

    assert_eq!(summary.total.tokens_used, 100);
    assert_eq!(summary.total.finished_runs_count, 2);
    assert_eq!(summary.this_week.tokens_used, 10);
    assert_eq!(summary.this_week.input_tokens, 4);
    assert_eq!(summary.this_week.finished_runs_count, 1);
}

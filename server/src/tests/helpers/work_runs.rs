use super::*;

pub async fn insert_pending_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    teams::ensure_default_team(pool).await;
    insert_pending_work_run_for_team(pool, DEFAULT_TEAM_ID, project_config_id, task_ref).await
}

pub async fn insert_pending_work_run_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        teams::ensure_default_team(pool).await;
    }

    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        team_id,
        external_task_ref: task_ref.to_owned(),
        task_title: None,
        task_slug: None,
        project_config_id,
        repo_full_names: Vec::new(),
        status: WorkRunStatus::Pending,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
        github_installation_id: None,
        github_delivery_id: None,
    };

    repo.insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id
}

pub async fn insert_running_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
    worker_id: Uuid,
) -> Uuid {
    teams::ensure_default_team(pool).await;
    insert_running_work_run_for_team(
        pool,
        DEFAULT_TEAM_ID,
        project_config_id,
        task_ref,
        worker_id,
    )
    .await
}

pub async fn insert_running_work_run_for_team(
    pool: &sqlx::PgPool,
    team_id: Uuid,
    project_config_id: Uuid,
    task_ref: &str,
    worker_id: Uuid,
) -> Uuid {
    if team_id == DEFAULT_TEAM_ID {
        teams::ensure_default_team(pool).await;
    }

    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        team_id,
        external_task_ref: task_ref.to_owned(),
        task_title: None,
        task_slug: None,
        project_config_id,
        repo_full_names: Vec::new(),
        status: WorkRunStatus::Running,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
        github_installation_id: None,
        github_delivery_id: None,
    };
    let id = repo
        .insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id;

    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        id,
    )
    .execute(pool)
    .await
    .expect("Should set worker_id");

    sqlx::query!(
        "UPDATE workers SET active_jobs = active_jobs + 1, status = 'busy'::worker_status WHERE id = $1",
        worker_id,
    )
    .execute(pool)
    .await
    .expect("Should reserve worker capacity");

    id
}

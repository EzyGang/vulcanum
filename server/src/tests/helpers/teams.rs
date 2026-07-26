use super::{Uuid, DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE, DEFAULT_TEAM_ID};

pub async fn ensure_default_team(pool: &sqlx::PgPool) {
    sqlx::query!(
        r#"INSERT INTO teams (id, name, prompt_template, review_prompt_template)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (id) DO UPDATE
           SET name = EXCLUDED.name,
               prompt_template = EXCLUDED.prompt_template,
               review_prompt_template = EXCLUDED.review_prompt_template"#,
        DEFAULT_TEAM_ID,
        "Default team",
        "",
        "",
    )
    .execute(pool)
    .await
    .expect("Should ensure default team");
}

pub async fn insert_team(pool: &sqlx::PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO teams (id, name, prompt_template, review_prompt_template)
           VALUES ($1, $2, $3, $4)"#,
        id,
        name,
        DEFAULT_PROMPT_TEMPLATE,
        DEFAULT_REVIEW_PROMPT_TEMPLATE,
    )
    .execute(pool)
    .await
    .expect("Should insert team");

    id
}

pub async fn insert_user(pool: &sqlx::PgPool, id: &str) {
    let email = format!("{id}@example.com");

    sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", id, email)
        .execute(pool)
        .await
        .expect("Should insert user");
}

use sqlx::PgPool;

#[sqlx::test]
async fn schema_tables_exist(pool: PgPool) {
    let tables: Vec<String> = sqlx::query_scalar(
        "select table_name::text from information_schema.tables
         where table_schema = 'public' and table_name = any($1)",
    )
    .bind(
        [
            "users",
            "api_tokens",
            "tech_stacks",
            "software_items",
            "stack_battles",
        ]
        .as_slice(),
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(tables.len(), 5, "missing tables: {tables:?}");
}

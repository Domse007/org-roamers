use sqlx::SqlitePool;

pub mod files;
pub mod init;
pub mod olp;
pub mod rebuild;

pub async fn init_db(_strict: bool) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;

    init::init_files_table(&pool).await?;
    init::init_nodes_table(&pool).await?;
    init::init_links_table(&pool).await?;
    init::init_aliases(&pool).await?;
    init::init_tags(&pool).await?;
    init::init_olp_table(&pool).await?;

    Ok(pool)
}

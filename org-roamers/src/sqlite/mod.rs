use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub mod files;
pub mod init;
pub mod olp;
pub mod rebuild;

pub async fn init_db() -> anyhow::Result<SqlitePool> {
    // FIXME: issue https://github.com/launchbadge/sqlx/issues/2510
    let pool_options = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None);

    let pool = pool_options
        .connect("sqlite:file:org-roamers-db?mode=memory&cache=shared")
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    init::init_files_table(&pool).await?;
    init::init_nodes_table(&pool).await?;
    init::init_links_table(&pool).await?;
    init::init_aliases(&pool).await?;
    init::init_tags(&pool).await?;
    init::init_olp_table(&pool).await?;

    Ok(pool)
}

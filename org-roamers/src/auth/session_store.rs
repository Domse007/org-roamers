use sqlx::SqlitePool;
use tower_sessions_sqlx_store::SqliteStore;

/// Initialize session store using the shared SqlitePool
/// The session store will create its own table (tower_sessions) in the same database
pub async fn create_session_store(pool: SqlitePool) -> anyhow::Result<SqliteStore> {
    use tracing::info;

    // Create store from existing pool
    let store = SqliteStore::new(pool);

    // Run migrations to create session table
    info!("Running session store migrations...");
    store.migrate().await?;
    info!("Session store initialized");

    Ok(store)
}

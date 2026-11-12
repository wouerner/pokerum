use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Abre o pool, habilita foreign keys/WAL e roda as migrations.
pub async fn connect(url: &str) -> anyhow::Result<SqlitePool> {
    let in_memory = url.contains(":memory:") || url.contains("mode=memory");
    let mut opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true);
    if !in_memory {
        opts = opts
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));
    }

    let pool_opts = if in_memory {
        // Cada conexão em memória é um banco separado: usar uma única conexão eterna.
        SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
    } else {
        SqlitePoolOptions::new().max_connections(8)
    };

    let pool = pool_opts.connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

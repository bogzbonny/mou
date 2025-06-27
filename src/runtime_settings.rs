//! Runtime settings are persisted settings that are used to configure the behavior of the application at runtime.
//!
//! Unlike the configuration, these can during runtime, and are only intended to be used for
//! internal operation of kwaak.

use {
    crate::{duckdb_index::get_duckdb, Config},
    anyhow::{Context, Result},
    serde::{Deserialize, Serialize},
    swiftide::integrations::duckdb::Duckdb,
    tokio::sync::RwLock,
};

pub struct RuntimeSettings {
    db: Duckdb,
    schema_created: RwLock<bool>,
}

impl RuntimeSettings {
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let db = get_duckdb(config);

        Self {
            db,
            schema_created: false.into(),
        }
    }

    #[must_use]
    pub fn from_db(db: Duckdb) -> Self {
        Self {
            db,
            schema_created: false.into(),
        }
    }

    #[must_use]
    pub async fn get<VALUE: for<'a> Deserialize<'a>>(&self, key: &str) -> Option<VALUE> {
        self.lazy_create_schema().await.ok()?;

        let conn = self.db.connection().lock().unwrap();
        let sql = "SELECT value FROM runtime_settings WHERE key = ?";

        serde_json::from_str(
            &conn
                .query_row(sql, [key], |row| row.get::<_, String>(0))
                .ok()?,
        )
        .ok()
    }

    pub async fn set<VALUE: Serialize>(&self, key: &str, value: VALUE) -> Result<()> {
        self.lazy_create_schema().await?;
        let conn = self.db.connection().lock().unwrap();
        let sql = "INSERT OR REPLACE INTO runtime_settings (key, value) VALUES (?, ?)";

        conn.execute(sql, [key, &serde_json::to_string(&value)?])
            .context("Failed to set runtime setting")?;

        Ok(())
    }

    async fn lazy_create_schema(&self) -> Result<()> {
        if *self.schema_created.read().await {
            return Ok(());
        }
        let mut lock = self.schema_created.write().await;

        let sql = "CREATE TABLE IF NOT EXISTS runtime_settings (key TEXT PRIMARY KEY, value TEXT)";
        let conn = self.db.connection().lock().unwrap();
        conn.execute(sql, [])
            .context("Failed to create runtime settings table")?;

        *lock = true;
        Ok(())
    }
}

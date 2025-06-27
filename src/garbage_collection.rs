//! This module identifies statements changed since the last index date and removes them from the index.
//!
//!
//! NOTE: If more general settings are added to duckdb, better extract this to a more general place.

use {
    super::duckdb_index,
    crate::{runtime_settings::RuntimeSettings, Config, Statement},
    anyhow::{Context as _, Result},
    std::{borrow::Cow, time::SystemTime},
    swiftide::{integrations::duckdb::Duckdb, traits::Persist},
};

const LAST_CLEANED_UP_AT: &str = "last_cleaned_up_at";

#[derive(Debug)]
pub struct GarbageCollector<'config> {
    /// The last index date
    config: Cow<'config, Config>,
    duckdb: Duckdb,
    /// Extensions to consider for GC
    file_extensions: Vec<&'config str>,
}

impl<'config> GarbageCollector<'config> {
    pub fn from_config(config: &'config Config) -> Self {
        let mut file_extensions = config.language_extensions();
        file_extensions.push("md");

        Self {
            config: Cow::Borrowed(config),
            duckdb: duckdb_index::get_duckdb(config),
            file_extensions,
        }
    }

    fn runtime_settings(&self) -> RuntimeSettings {
        // TODO: Bit of a code smell, maybe just pass it around from the config instead
        // singleton is painful
        if cfg!(test) {
            RuntimeSettings::from_db(self.duckdb.clone())
        } else {
            self.config.runtime_settings()
        }
    }

    async fn get_last_cleaned_up_at(&self) -> Option<SystemTime> {
        self.runtime_settings().get(LAST_CLEANED_UP_AT).await
    }

    async fn update_last_cleaned_up_at(&self, date: SystemTime) {
        if let Err(e) = self.runtime_settings().set(LAST_CLEANED_UP_AT, date).await {
            tracing::error!("Failed to update last cleaned up at: {:#}", e);
        }
    }

    // NOTE this is a placeholder
    async fn statements_deleted_since_last_index(&self) -> Vec<Statement> {
        return vec![];
    }

    // NOTE this is a placeholder
    async fn statements_changed_since_last_index(&self) -> Vec<Statement> {
        return vec![];
    }

    async fn delete_statements_from_index(&self, statements: Vec<Statement>) -> Result<()> {
        // Ensure the table is set up
        tracing::info!(
            "Setting up duckdb table for deletion of statements: {:?}",
            statements
        );
        if let Err(err) = self.duckdb.setup().await {
            // Duck currently does not allow `IF NOT EXISTS` on creating indices.
            // We just ignore the error here if the table already exists.
            // This is expected to happen always.
            tracing::debug!("Failed to setup duckdb in GC (this is ok): {:#}", err);
        }

        let mut conn = self.duckdb.connection().lock().unwrap();
        let tx = conn.transaction()?;

        {
            let table = self.duckdb.table_name();
            let mut stmt = tx.prepare(&format!("DELETE FROM {table} WHERE path = ?"))?;

            for statement in statements {
                tracing::debug!(?statement, "Deleting file from Duckdb index with predicate",);
                stmt.execute([statement.to_string()])?;
            }
        }
        tx.commit()?;

        Ok(())
    }

    fn delete_statements_from_cache(&self, statements: &[Statement]) -> Result<()> {
        tracing::info!("Deleting statements from cache: {:?}", statements);

        let mut conn = self.duckdb.connection().lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(&format!(
                "DELETE FROM {} WHERE path = ?",
                self.duckdb.cache_table()
            ))?;

            for statement in statements {
                tracing::debug!("Removing node from cache: {}", statement.to_string());
                stmt.execute([statement.to_string()])
                    .context("failed to remove file from cache")?;
            }
        }
        tx.commit()?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn clean_up(&self) -> Result<()> {
        // Introduce logging for step-by-step tracing
        tracing::info!("Starting cleanup process.");

        let statements = [
            self.statements_changed_since_last_index().await,
            self.statements_deleted_since_last_index().await,
        ]
        .concat();

        if statements.is_empty() {
            tracing::info!("No statements changed since last index; skipping garbage collection");
            self.update_last_cleaned_up_at(SystemTime::now()).await;
            return Ok(());
        }

        if self.never_been_indexed().await {
            tracing::warn!("No index date found; skipping garbage collection");
            self.update_last_cleaned_up_at(SystemTime::now()).await;
            return Ok(());
        }

        tracing::warn!(
            "Found {} changed/deleted statements since last index; garbage collecting ...",
            statements.len()
        );

        tracing::debug!(?statements, "Files changed since last index");

        {
            if let Err(e) = self.delete_statements_from_cache(&statements) {
                self.update_last_cleaned_up_at(SystemTime::now()).await;
                return Err(e);
            }

            if let Err(e) = self.delete_statements_from_index(statements).await {
                self.update_last_cleaned_up_at(SystemTime::now()).await;
                return Err(e);
            }
        }

        self.update_last_cleaned_up_at(SystemTime::now()).await;

        tracing::info!("Garbage collection completed and cleaned up at updated.");

        Ok(())
    }

    // Returns true if no rows were indexed, or otherwise errors were encountered
    #[tracing::instrument(skip(self))]
    async fn never_been_indexed(&self) -> bool {
        let conn = self.duckdb.connection().lock().unwrap();
        let table = self.duckdb.table_name();

        let num = conn.query_row_and_then(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get::<_, i64>(0)
        });

        if let Err(e) = &num {
            tracing::error!("Failed to determine if index has been done: {e:#}");
        }

        num.map(|n| n == 0).unwrap_or(true)
    }
}

use crate::{Driver, Transaction};

use futures::executor;
use sqlx::{
    sqlite::{Sqlite, SqliteConnection},
    Connection, Error,
};
use std::array::TryFromSliceError;
use std::convert::TryInto;

// SQL definition for the backing table
const MIGRATION_BACKING_DEF: &'static str = r#"
CREATE TABLE IF NOT EXISTS nomad_migrations (
    namespace   text not null primary key,
    version     blob not null
) WITHOUT ROWID;
"#;

fn ensure_migration_table(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    executor::block_on(sqlx::query(MIGRATION_BACKING_DEF).execute(conn)).and(Ok(()))
}

impl<'a> Transaction<'a, SqliteConnection> for sqlx::Transaction<'a, Sqlite> {
    fn commit(self) -> Result<(), <SqliteConnection as Driver<'a>>::Error> {
        executor::block_on(self.commit())
    }

    fn execute_sql(&mut self, sql: &str) -> Result<(), <SqliteConnection as Driver<'a>>::Error> {
        executor::block_on(sqlx::query(sql).execute(self)).and(Ok(()))
    }

    fn push_latest_version(
        &mut self,
        namespace: &str,
        version: u64,
    ) -> Result<(), <SqliteConnection as Driver<'a>>::Error> {
        ensure_migration_table(self)?;

        let version_bytes = &version.to_le_bytes()[..];

        executor::block_on(
            sqlx::query(
                "INSERT INTO nomad_migrations(namespace, version) VALUES(?, ?) ON CONFLICT(namespace) DO UPDATE SET version = excluded.version"
            )
            .bind(namespace)
            .bind(version_bytes)
            .execute(self)
        ).and(Ok(()))
    }
}

impl<'a> Driver<'a> for SqliteConnection {
    type Transaction = sqlx::Transaction<'a, Sqlite>;
    type Error = sqlx::Error;

    fn begin(&'a mut self) -> Result<Self::Transaction, Self::Error> {
        executor::block_on(Connection::begin(self))
    }

    fn latest_version(&mut self, namespace: &str) -> Result<Option<u64>, Self::Error> {
        fn map_conv_err(err: TryFromSliceError) -> Error {
            Error::ColumnDecode {
                index: "version".to_owned(),
                source: Box::new(err),
            }
        }

        ensure_migration_table(self)?;

        let version: Option<Vec<u8>> = futures::executor::block_on(
            sqlx::query_scalar("SELECT version FROM nomad_migrations WHERE namespace = ?")
                .bind(namespace)
                .fetch_optional(self),
        )?;

        Ok(match version {
            Some(vec) => Some(u64::from_le_bytes(
                vec[..].try_into().map_err(map_conv_err)?,
            )),
            None => None,
        })
    }
}

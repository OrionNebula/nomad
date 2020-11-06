use crate::{Driver, Transaction};

use rusqlite::{params, types::Type, Connection, Error, OptionalExtension, Row, NO_PARAMS};
use std::array::TryFromSliceError;
use std::convert::TryInto;

// SQL definition for the backing table
const MIGRATION_BACKING_DEF: &'static str = r#"
CREATE TABLE IF NOT EXISTS nomad_migrations (
    namespace   text not null primary key,
    version     blob not null
) WITHOUT ROWID;
"#;

// Ensure that the migration table exists for us to read from
fn ensure_migration_table(conn: &Connection) -> Result<(), Error> {
    conn.execute(MIGRATION_BACKING_DEF, NO_PARAMS).and(Ok(()))
}

impl<'a> Driver<'a> for Connection {
    type Transaction = rusqlite::Transaction<'a>;
    type Error = Error;

    fn begin(&'a mut self) -> Result<Self::Transaction, Self::Error> {
        self.transaction()
    }

    fn latest_version(&mut self, namespace: &str) -> Result<Option<u64>, Self::Error> {
        fn map_conv_err(err: TryFromSliceError) -> Error {
            Error::FromSqlConversionFailure(0, Type::Blob, Box::new(err))
        }

        fn convert_row(row: &Row) -> Result<u64, Error> {
            let blob = row.get_raw(0).as_blob()?.try_into().map_err(map_conv_err)?;

            Ok(u64::from_le_bytes(blob))
        }

        ensure_migration_table(self)?;

        self.query_row(
            "SELECT version FROM nomad_migrations WHERE namespace = ?",
            params![namespace],
            convert_row,
        )
        .optional()
    }
}

impl<'a> Transaction<'a, Connection> for rusqlite::Transaction<'a> {
    fn commit(self) -> Result<(), Error> {
        self.commit()
    }

    fn execute_sql(&mut self, sql: &str) -> Result<(), Error> {
        self.execute_batch(sql)
    }

    fn push_latest_version(&mut self, namespace: &str, version: u64) -> Result<(), Error> {
        ensure_migration_table(self)?;

        let version_bytes = &version.to_le_bytes()[..];

        self.execute(
            "INSERT INTO nomad_migrations(namespace, version) VALUES(?, ?) ON CONFLICT(namespace) DO UPDATE SET version = excluded.version",
            params![
                namespace,
                version_bytes
            ]
        ).and(Ok(()))
    }
}

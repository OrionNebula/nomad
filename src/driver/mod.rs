#[cfg(feature = "rusqlite")]
mod rusqlite;

#[cfg(feature = "sqlx")]
mod sqlx;

use std::error::Error;

pub trait Driver<'a> where Self: Sized, Self::Error: Error, Self::Transaction: Transaction<'a, Self> {
    type Transaction;
    type Error;

    // Begin a transaction
    fn begin(&'a mut self) -> Result<Self::Transaction, Self::Error>;

    // Get the latest migrated version for a given namespace
    fn latest_version(&mut self, namespace: &str) -> Result<Option<u64>, Self::Error>;
}

pub trait Transaction<'a, D: Driver<'a>> {
    // Commit the changes made during this transaction's life
    fn commit(self) -> Result<(), <D as Driver<'a>>::Error>;

    // Execute arbitrary SQL in the context of this transaction
    fn execute_sql(&mut self, sql: &str) -> Result<(), <D as Driver<'a>>::Error>;

    // Update the latest migrated version for a given namespace
    fn push_latest_version(&mut self, namespace: &str, version: u64) -> Result<(), <D as Driver<'a>>::Error>;
}

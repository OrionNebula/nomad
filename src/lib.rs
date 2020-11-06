extern crate nomad_macro;
pub use nomad_macro::*;

#[cfg(feature = "rusqlite")]
extern crate rusqlite;

#[cfg(feature = "sqlx")]
extern crate futures;
#[cfg(feature = "sqlx")]
extern crate sqlx;

mod driver;
mod error;
mod migration;
pub mod ordered;

pub use driver::{Driver, Transaction};
pub use error::*;
pub use migration::*;

pub const DEFAULT_NAMESPACE: &'static str = "nomad";

pub struct MigrationRunner<'d, 'n, D: Driver<'d>> {
    pub driver: &'d mut D,
    pub namespace: &'n str,
}

impl<'d, D: Driver<'d>> MigrationRunner<'d, 'static, D> {
    pub fn new(driver: &'d mut D) -> Self {
        MigrationRunner {
            driver,
            namespace: DEFAULT_NAMESPACE,
        }
    }
}

impl<'d, 'n, D: Driver<'d>> MigrationRunner<'d, 'n, D> {
    pub fn with_namespace(driver: &'d mut D, namespace: &'n str) -> Self {
        MigrationRunner { driver, namespace }
    }

    pub fn migrate<
        'a,
        T: AsRef<[Migration<'a>]>,
        C: Into<ordered::OrderedArray<Migration<'a>, T>>,
    >(
        self,
        migrations: C,
    ) -> Result<Option<u64>, MigrationError<D::Error>> {
        let latest_version = self.driver.latest_version(self.namespace)?;

        let mut txn = self.driver.begin()?;

        let mut last_executed = None;
        let mut latest_observed = None;
        for migration in &migrations.into() {
            latest_observed = Some(migration.version);

            match latest_version {
                Some(version) if migration.version <= version => continue,
                _ => {}
            }

            txn.execute_sql(migration.sql)?;
            txn.push_latest_version(self.namespace, migration.version)?;

            last_executed = Some(migration.version);
        }

        if let Some(latest_observed) = latest_observed {
            if let Some(latest_version) = latest_version {
                if latest_version > latest_observed {
                    return Err(MigrationError::TimeTravelError);
                }
            }
        }

        txn.commit()?;

        Ok(last_executed)
    }
}

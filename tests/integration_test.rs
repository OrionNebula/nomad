extern crate nomad;

macro_rules! driver_tests {
    ($driver:expr) => {
        #[test]
        fn runtime_migrate() {
            let migrations = [
                ::nomad::Migration {
                    version: 1,
                    sql: "-- test migration 1",
                },
                ::nomad::Migration {
                    version: 2,
                    sql: "-- test migration 2",
                },
            ]
            .into();

            let mut driver = $driver;

            ::nomad::MigrationRunner::new(&mut driver)
                .migrate(&migrations)
                .expect("Migrations should succeed")
                .expect("Migrations should be executed");

            assert_eq!(
                ::nomad::MigrationRunner::new(&mut driver)
                    .migrate(&migrations)
                    .expect("Migrations should succeed"),
                None
            );

            ::nomad::MigrationRunner::with_namespace(&mut driver, "test")
                .migrate(&migrations)
                .expect("Migrations should succeed")
                .expect("Migrations in another namespace should be executed");
        }

        #[test]
        fn compile_migrate() {
            let migrations = ::nomad::nomad_migrations!("./tests/migrations");

            let mut driver = $driver;

            ::nomad::MigrationRunner::new(&mut driver)
                .migrate(&migrations)
                .expect("Migrations should succeed")
                .expect("Migrations should be executed");

            assert_eq!(
                ::nomad::MigrationRunner::new(&mut driver)
                    .migrate(&migrations)
                    .expect("Migrations should succeed"),
                None
            );

            ::nomad::MigrationRunner::with_namespace(&mut driver, "test")
                .migrate(&migrations)
                .expect("Migrations should succeed")
                .expect("Migrations in another namespace should be executed");
        }

        #[test]
        fn time_travel() {
            let migrations = [
                ::nomad::Migration {
                    version: 1,
                    sql: "-- test migration 1",
                },
                ::nomad::Migration {
                    version: 2,
                    sql: "-- test migration 2",
                },
            ];

            let mut driver = $driver;

            ::nomad::MigrationRunner::new(&mut driver)
                .migrate(migrations)
                .expect("Migrations should succeed")
                .expect("Migrations should be executed");

            let migrations = [::nomad::Migration {
                version: 1,
                sql: "-- test migration 1",
            }];

            match ::nomad::MigrationRunner::new(&mut driver).migrate(migrations) {
                Err(::nomad::MigrationError::TimeTravelError) => {}
                _ => panic!("Expected a TimeTravelError"),
            }
        }

        #[test]
        fn rollback() {
            use ::nomad::Driver;

            let migrations = [
                ::nomad::Migration {
                    version: 1,
                    sql: "-- test migration 1",
                },
                ::nomad::Migration {
                    version: 2,
                    sql: "evil",
                },
                ::nomad::Migration {
                    version: 2,
                    sql: "-- test migration 2",
                },
            ];

            let mut driver = $driver;

            if let Ok(_) = ::nomad::MigrationRunner::new(&mut driver).migrate(migrations) {
                panic!("Migrations should fail");
            }

            assert_eq!(
                driver
                    .latest_version(::nomad::DEFAULT_NAMESPACE)
                    .expect("Should be able to get a version"),
                None
            );
        }
    };
}

mod dummy {
    use std::collections::HashMap;
    use std::fmt::Display;

    #[derive(Debug)]
    struct DummyError();

    impl Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl std::error::Error for DummyError {}

    #[derive(Default)]
    struct DummyDriver {
        latest_versions: HashMap<String, u64>,
    }

    struct DummyTransaction<'a> {
        driver: &'a mut DummyDriver,
        changes: HashMap<String, u64>,
    }

    impl ::nomad::Transaction<'_, DummyDriver> for DummyTransaction<'_> {
        fn commit(self) -> Result<(), DummyError> {
            for (k, v) in self.changes {
                self.driver.latest_versions.insert(k, v);
            }

            Ok(())
        }

        fn execute_sql(&mut self, sql: &str) -> Result<(), DummyError> {
            if sql == "evil" {
                Err(DummyError())
            } else {
                Ok(())
            }
        }

        fn push_latest_version(&mut self, namespace: &str, version: u64) -> Result<(), DummyError> {
            self.changes.insert(namespace.to_owned(), version);

            Ok(())
        }
    }

    impl<'a> ::nomad::Driver<'a> for DummyDriver {
        type Transaction = DummyTransaction<'a>;
        type Error = DummyError;

        fn begin(&'a mut self) -> Result<Self::Transaction, Self::Error> {
            Ok(DummyTransaction {
                driver: self,
                changes: Default::default(),
            })
        }

        fn latest_version(&mut self, namespace: &str) -> Result<Option<u64>, Self::Error> {
            Ok(self.latest_versions.get(&namespace.to_owned()).map(|n| *n))
        }
    }

    driver_tests!(DummyDriver::default());
}

#[cfg(feature = "sqlx")]
mod sqlx {
    #[cfg(feature = "sqlx-sqlite")]
    mod sqlite {
        use ::sqlx::sqlite::SqliteConnection;
        use ::sqlx::Connection;
        use futures::executor::block_on;

        driver_tests!(block_on(SqliteConnection::connect("sqlite::memory:"))
            .expect("Falied to open an in-memory SQLite database"));
    }
}

#[cfg(feature = "rusqlite")]
mod rusqlite {
    use ::rusqlite::Connection;

    driver_tests!(
        Connection::open_in_memory().expect("Failed to open an in-memory SQLite database")
    );
}

mod ordered {
    use ::nomad::{Migration, OrderedMigrations};

    #[test]
    fn into() {
        const MIGRATIONS: [Migration; 2] = [
            Migration {
                version: 2,
                sql: "-- test migration 2",
            },
            Migration {
                version: 1,
                sql: "-- test migration 1",
            },
        ];

        let arr: OrderedMigrations<_> = MIGRATIONS.into();

        assert_eq!(
            arr.into_iter().map(|m| m.version).collect::<Vec<u64>>(),
            [1, 2]
        )
    }

    #[test]
    fn try_new() {
        const SORTED_MIGRATIONS: [Migration; 2] = [
            Migration {
                version: 1,
                sql: "-- test migration 1",
            },
            Migration {
                version: 2,
                sql: "-- test migration 2",
            },
        ];

        const UNSORTED_MIGRATIONS: [Migration; 2] = [
            Migration {
                version: 2,
                sql: "-- test migration 2",
            },
            Migration {
                version: 1,
                sql: "-- test migration 1",
            },
        ];

        match OrderedMigrations::try_new(&SORTED_MIGRATIONS) {
            None => panic!("Sorted migrations failed to create"),
            _ => {}
        }

        match OrderedMigrations::try_new(&UNSORTED_MIGRATIONS) {
            Some(_) => panic!("Unsorted migrations created as sorted"),
            _ => {}
        }
    }
}

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum MigrationError<E: Error> {
    TimeTravelError,
    DriverError(E),
}

impl<E: Error> Display for MigrationError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeTravelError => write!(
                f,
                "Latest migrated version exceeds latest known version - possible downgrade"
            ),
            Self::DriverError(err) => Display::fmt(err, f),
        }
    }
}

impl<E: Error> Error for MigrationError<E> {}

impl<E: Error> From<E> for MigrationError<E> {
    fn from(err: E) -> Self {
        Self::DriverError(err)
    }
}

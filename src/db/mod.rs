use std::sync::Mutex;

use sqlite::Connection;

pub mod models;

pub struct Database {
    connection: Mutex<Connection>,
}

#[derive(Debug)]
pub enum DbError {
    SQLite(sqlite::Error),
    PoisonedLock, // Probably want to just transparently recreate connection if this happens
}

impl From<sqlite::Error> for DbError {
    fn from(e: sqlite::Error) -> Self {
        DbError::SQLite(e)
    }
}

impl<T> From<std::sync::PoisonError<T>> for DbError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        DbError::PoisonedLock
    }
}

pub type Result<T> = std::result::Result<T, DbError>;

impl Database {
    pub fn new(db: &str) -> Self {
        let connection = sqlite::open(db).unwrap();

        // TODO: make flags and names unique
        connection
            .execute(
                "
                CREATE TABLE IF NOT EXISTS challenges (id INTEGER PRIMARY KEY, name TEXT, flag TEXT);
                ",
            )
            .unwrap();

        Database {
            connection: Mutex::new(connection),
        }
    }
}

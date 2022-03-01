use std::sync::Mutex;

use sqlite::{Connection, State};

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
    pub fn new() -> Self {
        let connection = sqlite::open(":memory:").unwrap();

        connection
            .execute(
                "
                CREATE TABLE IF NOT EXISTS challenges (id INTEGER, name TEXT, flag TEXT);
                ",
            )
            .unwrap();

        connection
            .execute(
                "
                INSERT INTO challenges VALUES (1, 'Hello, World!', 'cheri{helloworld}');
                ",
            )
            .unwrap();

        Database {
            connection: Mutex::new(connection),
        }
    }

    pub fn check_flag(&self, flag: &str) -> Result<Option<String>> {
        let connection = self.connection.lock()?;

        let mut statement = connection.prepare("SELECT name FROM challenges WHERE flag = ?")?;

        statement.bind(1, flag).unwrap();

        if let State::Row = statement.next()? {
            let challenge_name = statement.read::<String>(0).unwrap();

            Ok(Some(challenge_name))
        } else {
            Ok(None)
        }
    }

    pub fn add_challenge(&self, name: &str, flag: &str) -> Result<()> {
        let connection = self.connection.lock()?;

        let mut statement =
            connection.prepare("INSERT INTO challenges (name, flag) VALUES (?, ?)")?;

        statement.bind(1, name).unwrap();
        statement.bind(2, flag).unwrap();

        while State::Done != statement.next()? {}

        Ok(())
    }
}

use sqlite::State;

use super::{Database, Result};

pub struct Challenge {
    pub id: i64,
    pub name: String,
    pub flag: String,
}

impl Challenge {
    /// Creates a new challenge row in the database, returning the id.
    pub fn create_challenge(db: &Database, name: &str, flag: &str) -> Result<i64> {
        let connection = db.connection.lock()?;

        let mut stmt1 = connection.prepare("INSERT INTO challenges (name, flag) VALUES (?, ?);")?;
        stmt1.bind(1, name)?;
        stmt1.bind(2, flag)?;

        assert_eq!(stmt1.next()?, State::Done);

        let mut stmt2 = connection.prepare("SELECT last_insert_rowid();")?;

        assert_eq!(stmt2.next()?, State::Row);

        let id: i64 = stmt2.read(0).unwrap();

        assert_eq!(stmt2.next()?, State::Done);

        Ok(id)
    }

    /// Fetches a single challenge row from the database keyed by flag.
    pub fn get_by_flag(db: &Database, flag: &str) -> Result<Option<Self>> {
        let connection = db.connection.lock()?;

        let mut statement =
            connection.prepare("SELECT id, name, flag FROM challenges WHERE flag = ?")?;

        statement.bind(1, flag).unwrap();

        if let State::Row = statement.next()? {
            Ok(Some(Self {
                id: statement.read(0).unwrap(),
                name: statement.read(1).unwrap(),
                flag: statement.read(2).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }
}

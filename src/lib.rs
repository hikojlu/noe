use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Handle {
    conn: Connection,
}

impl Handle {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                number INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                done INTEGER NOT NULL )",
            [],
        )?;
        Ok(Self { conn })
    }
    pub fn list_notes(&self) -> Result<Notes> {
        let mut stmt = self.conn.prepare("SELECT number, text, done FROM notes")?;
        let row_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, u16>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
            ))
        })?;

        let notes = row_iter
            .map(|row| {
                let (number, text, done) = row?;
                Ok(Note { number, text, done })
            })
            .collect::<Result<Vec<Note>>>()?;

        Ok(Notes { notes })
    }
    pub fn new_note(&self, text: String, number: Option<u16>) -> Result<u16> {
        let number = {
            let mut stmt = self.conn.prepare("SELECT number FROM notes")?;

            let nums = stmt
                .query_map([], |row| row.get::<_, u16>(0))?
                .collect::<Result<Vec<u16>>>()?;

            let from_db = || nums.iter().max().unwrap_or(&0) + 1;

            let number =
                number.map_or_else(from_db, |n| if nums.contains(&n) { from_db() } else { n });

            number
        };

        self.conn.execute(
            "INSERT INTO notes(number, text, done) VALUES(?1, ?2, ?3)",
            params![number, text, false],
        )?;
        Ok(number)
    }
    pub fn done_note(&self, number: u16) -> Result<()> {
        self.conn.execute(
            "UPDATE notes SET done = ?1 WHERE number = ?2",
            params![true, number],
        )?;
        Ok(())
    }
    pub fn undone_note(&self, number: u16) -> Result<()> {
        self.conn.execute(
            "UPDATE notes SET done = ?1 WHERE number = ?2",
            params![false, number],
        )?;
        Ok(())
    }
    pub fn remove_note(&self, number: u16) -> Result<()> {
        self.conn
            .execute("DELETE FROM notes WHERE number = ?1", params![number])?;
        Ok(())
    }
}

pub struct Notes {
    notes: Vec<Note>,
}

impl Notes {
    pub fn as_vec(&self) -> &Vec<Note> {
        &self.notes
    }
}

pub struct Note {
    pub number: u16,
    pub text: String,
    pub done: bool,
}

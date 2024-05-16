use std::sync::Arc;

use rusqlite::{Connection, params};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::notebook::db::Documentable;
use crate::notebook::note::Note;

pub struct NotebookRepository {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug)]
struct Category {
    id: u32,
    label: String,
}

impl NotebookRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        NotebookRepository {
            conn
        }
    }

    pub async fn add_note(&self, note: &Note) -> Result<(), NotebookRepositoryError> {
        let conn = self.conn.lock().await;
        log::info!("Adding note {} to models db", note.id());
        conn.execute(
            "INSERT INTO notes (id, content) VALUES (?1, ?2)",
            (&note.id(), &note.text()),
        )?;
        Ok(())
    }

    pub async fn update_note(&self, note: &Note) -> Result<(), NotebookRepositoryError> {
        let conn = self.conn.lock().await;
        log::info!("Updating note {} in models db", note.id());
        conn.execute(
            "UPDATE notes SET content = ?1 WHERE id = ?2",
            (&note.text(), &note.id()),
        )?;
        Ok(())
    }

    pub async fn delete_note(&self, id: &str) -> Result<(), NotebookRepositoryError> {
        let conn = self.conn.lock().await;
        log::info!("Deleting note {} in models db", id);
        conn.execute(
            "DELETE FROM notes WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }


    pub async fn init_db(&self) -> Result<(), NotebookRepositoryError> {
        log::info!("Initializing models Db");
        // Execute a SQL statement to create a new table named "person".
        // The table includes an id (integer primary key), name (text), and data (blob) fields.
        let conn = self.conn.lock().await;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label  NVARCHAR(128) NOT NULL
        )", ())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
            id CHAR(15) PRIMARY KEY,
            content TEXT
        )", ())?;


        conn.execute("
            CREATE TABLE IF NOT EXISTS note_category (
            note_id CHAR(15),
            category_name NVARCHAR(128),
            PRIMARY KEY (note_id, category_name),
            FOREIGN KEY (note_id) REFERENCES notes (id),
            FOREIGN KEY (category_name) REFERENCES categories (name)
        )", ())?;

        Ok(())
    }
}

#[derive(Error, Debug, Serialize)]
pub enum NotebookRepositoryError {
    #[error("Models db error: {0}")]
    ModelPersistence(String),

}

// We could use the rusqlite::Error to create our own error variant
impl From<rusqlite::Error> for NotebookRepositoryError {
    fn from(e: rusqlite::Error) -> Self {
        NotebookRepositoryError::ModelPersistence(e.to_string())
    }
}
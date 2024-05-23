use std::sync::Arc;

use chrono::Utc;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;

use crate::notebook::note::Note;
use crate::notebook::NotebookError;

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

    pub async fn get_note(&self, id: &str) -> Result<Option<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Getting note {} from models db", id);
        let mut stmt = conn.prepare("SELECT id, content, created, modified FROM notes WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let text: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;

            Ok(Some(Note::hydrate(&id, &text, created, modified)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_notes_by_ids(&self, ids: Vec<&str>) -> Result<Vec<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Getting notes [{:?}] from models db", ids);
        // Construct the SQL query with the correct number of placeholders
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("SELECT id, content, created, modified FROM notes WHERE id IN ({})", placeholders);
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(ids.iter()))?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;
            notes.push(Note::hydrate(&id, &content, created, modified));
        }
        if notes.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(notes)
        }
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Getting all notes from models db");
        let mut stmt = conn.prepare("SELECT id, content, created, modified FROM notes")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let text: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;
            Ok(Note::hydrate(&id, &text, created, modified))
        })?;

        let notes: Vec<Note> = rows.collect::<Result<_, _>>()?;
        Ok(notes)
    }


    pub async fn add_note(&self, note: &Note) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Adding note {} to models db", note.get_id());
        conn.execute(
            "INSERT INTO notes (id, content, created, modified) VALUES (?1, ?2, ?3, ?4)",
            (&note.get_id(), &note.get_text(), &note.get_created(), &note.get_modified()),
        )?;
        Ok(())
    }

    pub async fn update_note(&self, note: &mut Note) -> Result<Note, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Updating note {} in models db", note.get_id());
        note.set_modified(Self::get_now());
        conn.execute(
            "UPDATE notes SET content = ?1, modified = ?2 WHERE id = ?3",
            (&note.get_text(), &note.get_modified(), &note.get_id()),
        )?;
        Ok(note.clone())
    }

    pub async fn delete_note(&self, id: &str) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Deleting note {} in models db", id);
        conn.execute(
            "DELETE FROM notes WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub async fn delete_all_notes(&self) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Deleting all notes");
        conn.execute(
            "DELETE FROM notes",
            params![],
        )?;
        Ok(())
    }


    pub async fn init_db(&self) -> Result<(), NotebookError> {
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
            id CHAR(36) PRIMARY KEY,
            content TEXT,
            created INTEGER NOT NULL,
            modified INTEGER NOT NULL
        )", ())?;


        conn.execute("
            CREATE TABLE IF NOT EXISTS note_category (
            note_id CHAR(36),
            category_name NVARCHAR(128),
            PRIMARY KEY (note_id, category_name),
            FOREIGN KEY (note_id) REFERENCES notes (id),
            FOREIGN KEY (category_name) REFERENCES categories (name)
        )", ())?;

        Ok(())
    }

    fn get_now() -> i64 {
        Utc::now().timestamp()
    }
}
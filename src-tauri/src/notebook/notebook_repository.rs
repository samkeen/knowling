use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use log::info;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;

use crate::notebook::note::{Category, Note};
use crate::notebook::NotebookError;

pub struct NotebookRepository {
    conn: Arc<Mutex<Connection>>,
}

impl NotebookRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        NotebookRepository { conn }
    }

    pub async fn get_note(&self, id: &str) -> Result<Option<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        info!("Getting note {} from models db", id);
        let mut stmt = conn.prepare(
            "
        SELECT n.id, n.content, n.created, n.modified, c.id, c.label
        FROM notes n
        LEFT JOIN note_category nc ON n.id = nc.note_id
        LEFT JOIN categories c ON nc.category_id = c.id
        WHERE n.id = ?1
    ",
        )?;
        let mut rows = stmt.query(params![id])?;

        let mut categories = HashSet::new();
        let (id, text, created, modified) = if let Some(row) = rows.next()? {
            // Extract the note details from the first row
            let id: String = row.get(0)?;
            let text: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;

            // If the category ID is present, extract the category label and insert a new Category into the HashSet
            if let Ok(category_id) = row.get::<_, String>(4) {
                let category_label: String = row.get(5)?;
                categories.insert(Category::hydrate(&category_id, &category_label));
            }

            (id, text, created, modified)
        } else {
            // If no rows are returned, the note doesn't exist, so return None
            return Ok(None);
        };

        // Process any remaining rows to extract additional categories for the note
        while let Some(row) = rows.next()? {
            // The `while let` loop continues to iterate as long as there are more rows to process
            // Each iteration of the loop processes one row at a time

            // If the category ID is present in the current row, extract the category label and insert a new Category into the HashSet
            if let Ok(category_id) = row.get::<_, String>(4) {
                let category_label: String = row.get(5)?;
                categories.insert(Category::hydrate(&category_id, &category_label));
            }
        }

        // Create a new Note instance with the extracted details and categories, and return it wrapped in an Option
        Ok(Some(Note::hydrate(
            &id, &text, categories, created, modified,
        )))
    }

    pub async fn get_category_by_id(&self, id: &str) -> Result<Option<Category>, NotebookError> {
        let conn = self.conn.lock().await;
        info!("Getting category by ID: {}", id);

        let mut stmt = conn.prepare("SELECT id, label FROM categories WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let label: String = row.get(1)?;
            Ok(Some(Category::hydrate(&id, &label)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_notes_by_ids(&self, ids: Vec<&str>) -> Result<Vec<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Getting notes [{:?}] from models db", ids);

        // Create a string of placeholders for the SQL query based on the number of IDs
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        // Construct the SQL query with the placeholders
        let sql = format!(
            "
        SELECT n.id, n.content, n.created, n.modified, c.id, c.label
        FROM notes n
        LEFT JOIN note_category nc ON n.id = nc.note_id
        LEFT JOIN categories c ON nc.category_id = c.id
        WHERE n.id IN ({})
    ",
            placeholders
        );

        let mut stmt = conn.prepare(&sql)?;

        // Execute the query and map the rows to tuples
        let rows = stmt.query_map(rusqlite::params_from_iter(ids.iter()), |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;
            let category_id: Option<String> = row.get(4)?;
            let category_label: Option<String> = row.get(5)?;
            Ok((id, content, created, modified, category_id, category_label))
        })?;

        let mut notes = Vec::new();
        let mut current_note: Option<(String, String, i64, i64, HashSet<Category>)> = None;

        // Iterate over the rows and build the notes with their categories
        for row in rows {
            let (id, content, created, modified, category_id, category_label) = row?;

            // If there is a current note and its ID matches the current row's ID
            if let Some((curr_id, curr_content, curr_created, curr_modified, curr_categories)) =
                &mut current_note
            {
                if *curr_id == id {
                    // If the category ID and label are present, create a new Category and insert it into the current note's categories
                    if let (Some(cat_id), Some(cat_label)) = (category_id, category_label) {
                        curr_categories.insert(Category::hydrate(&cat_id, &cat_label));
                    }
                    continue; // Move to the next row
                } else {
                    // If the current row belongs to a different note, push the current note to the notes vector
                    notes.push(Note::hydrate(
                        curr_id,
                        curr_content,
                        std::mem::take(curr_categories),
                        *curr_created,
                        *curr_modified,
                    ));
                }
            }

            // Create a new HashSet for the categories of the current note
            let mut categories = HashSet::new();
            if let (Some(cat_id), Some(cat_label)) = (category_id, category_label) {
                categories.insert(Category::hydrate(&cat_id, &cat_label));
            }

            // Set the current note to the current row's data
            current_note = Some((id, content, created, modified, categories));
        }

        // If there is a remaining current note, push it to the notes vector
        if let Some((id, content, created, modified, categories)) = current_note {
            notes.push(Note::hydrate(&id, &content, categories, created, modified));
        }

        Ok(notes)
    }

    pub async fn get_or_create_category(&self, cat_label: &str) -> Result<Category, NotebookError> {
        let conn = self.conn.lock().await;
        let cat_label = cat_label.trim();
        info!("Upserting category '{}'", cat_label);

        // Perform a case-insensitive search for the category label
        let mut stmt =
            conn.prepare("SELECT id, label FROM categories WHERE LOWER(label) = LOWER(?1)")?;
        let mut rows = stmt.query(params![cat_label.to_lowercase()])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let db_label: String = row.get(1)?;
            // Return the category with the label from the database
            Ok(Category::hydrate(&id, &db_label))
        } else {
            // Category doesn't exist, create a new category
            let new_category = Category::new(cat_label);
            conn.execute(
                "INSERT INTO categories (id, label) VALUES (?1, ?2)",
                params![new_category.get_id(), new_category.get_label()],
            )?;
            Ok(new_category)
        }
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Getting all notes from models db");

        // Prepare the SQL query to fetch all notes with their categories
        let mut stmt = conn.prepare(
            "
        SELECT n.id, n.content, n.created, n.modified, c.id, c.label
        FROM notes n
        LEFT JOIN note_category nc ON n.id = nc.note_id
        LEFT JOIN categories c ON nc.category_id = c.id
    ",
        )?;

        // Execute the query and map the rows to tuples
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let created: i64 = row.get(2)?;
            let modified: i64 = row.get(3)?;
            let category_id: Option<String> = row.get(4)?;
            let category_label: Option<String> = row.get(5)?;
            Ok((id, content, created, modified, category_id, category_label))
        })?;

        let mut notes = Vec::new();
        let mut current_note: Option<(String, String, i64, i64, HashSet<Category>)> = None;

        // Iterate over the rows and build the notes with their categories
        for row in rows {
            let (id, content, created, modified, category_id, category_label) = row?;

            // If there is a current note and its ID matches the current row's ID
            if let Some((curr_id, curr_content, curr_created, curr_modified, curr_categories)) =
                &mut current_note
            {
                if *curr_id == id {
                    // If the category ID and label are present, create a new Category and insert it into the current note's categories
                    if let (Some(cat_id), Some(cat_label)) = (category_id, category_label) {
                        curr_categories.insert(Category::hydrate(&cat_id, &cat_label));
                    }
                    continue; // Move to the next row
                } else {
                    // If the current row belongs to a different note, push the current note to the notes vector
                    notes.push(Note::hydrate(
                        curr_id,
                        curr_content,
                        std::mem::take(curr_categories),
                        *curr_created,
                        *curr_modified,
                    ));
                }
            }

            // Create a new HashSet for the categories of the current note
            let mut categories = HashSet::new();
            if let (Some(cat_id), Some(cat_label)) = (category_id, category_label) {
                categories.insert(Category::hydrate(&cat_id, &cat_label));
            }

            // Set the current note to the current row's data
            current_note = Some((id, content, created, modified, categories));
        }

        // If there is a remaining current note, push it to the notes vector
        if let Some((id, content, created, modified, categories)) = current_note {
            notes.push(Note::hydrate(&id, &content, categories, created, modified));
        }

        Ok(notes)
    }

    pub async fn add_note(&self, note: &Note) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Adding note {} to models db", note.get_id());
        conn.execute(
            "INSERT INTO notes (id, content, created, modified) VALUES (?1, ?2, ?3, ?4)",
            (
                &note.get_id(),
                &note.get_text(),
                &note.get_created(),
                &note.get_modified(),
            ),
        )?;
        Ok(())
    }

    /// Updating a Note's text is quite frequent so there is a method just for that
    /// Updating anything associated with a note is separate (ex: add_category_to_note)
    pub async fn update_note_text(&self, note: &Note) -> Result<Note, NotebookError> {
        let conn = self.conn.lock().await;
        info!("Updating note {} in models db", note.get_id());
        // Update the note content and modified timestamp
        conn.execute(
            "UPDATE notes SET content = ?1, modified = ?2 WHERE id = ?3",
            (&note.get_text(), &note.get_modified(), &note.get_id()),
        )?;
        Ok(note.clone())
    }

    pub async fn reconcile_note_categories(&self, note: &Note) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        info!(
            "Reconciling Note [{}] categories {:?} in models db",
            note.get_id(),
            note.get_categories()
        );

        // Delete existing category associations for the note
        conn.execute(
            "DELETE FROM note_category WHERE note_id = ?1",
            params![note.get_id()],
        )?;
        // Insert new category associations for the note
        let categories: Vec<(&str, &str)> = note
            .get_categories()
            .iter()
            .map(|category| (note.get_id(), category.get_id()))
            .collect();
        // Build and execute a single INSERT query
        let placeholders = categories
            .iter()
            .map(|_| "(?, ?)")
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO note_category (note_id, category_id) VALUES {}",
            placeholders
        );

        let params = rusqlite::params_from_iter(
            categories
                .iter()
                .flat_map(|(note_id, category_id)| vec![note_id, category_id]),
        );

        conn.execute(&sql, params)?;
        Ok(())
    }

    pub async fn delete_note(&self, id: &str) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Deleting note {} in models db", id);
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub async fn delete_all_notes(&self) -> Result<(), NotebookError> {
        let conn = self.conn.lock().await;
        log::info!("Deleting all notes");
        conn.execute("DELETE FROM notes", params![])?;
        Ok(())
    }

    pub async fn init_db(&self) -> Result<(), NotebookError> {
        log::info!("Initializing models Db");
        // Execute a SQL statement to create a new table named "person".
        // The table includes an id (integer primary key), name (text), and data (blob) fields.
        let conn = self.conn.lock().await;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
            id CHAR(36) PRIMARY KEY,
            label  NVARCHAR(128) NOT NULL)", ())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
            id CHAR(36) PRIMARY KEY,
            content TEXT,
            created INTEGER NOT NULL,
            modified INTEGER NOT NULL)", ())?;

        conn.execute("CREATE TABLE IF NOT EXISTS note_category (
            note_id CHAR(36),
            category_id CHAR(36),
            PRIMARY KEY (note_id, category_id),
            FOREIGN KEY (note_id) REFERENCES notes (id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES categories (id)
        )", ())?;
        Ok(())
    }
}

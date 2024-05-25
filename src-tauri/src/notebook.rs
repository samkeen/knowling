use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use log::info;
use rusqlite::Connection;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;
use vec_embed_store::{EmbedDbError, EmbeddingEngineOptions, EmbeddingsDb, TextChunk};

use crate::notebook::note::{Category, Note};
use crate::notebook::notebook_repository::NotebookRepository;

pub mod note;
mod notebook_repository;

const SIMILARS_DEFAULT_LIMIT: usize = 3;
const SIMILARS_DEFAULT_THRESHOLD: f32 = 0.01;

pub struct Notebook {
    embed_store: EmbeddingsDb,
    models_store: NotebookRepository,
}

impl Notebook {
    pub async fn new(
        embedding_engine_options: EmbeddingEngineOptions,
        app_dir: &Path,
    ) -> Result<Self, NotebookError> {
        // cast app_dir to a &str
        let app_dir_str = app_dir.to_str().ok_or(NotebookError::FileAccess(
            "Invalid app directory path".to_string(),
        ))?;
        let embed_store = EmbeddingsDb::new(app_dir_str, embedding_engine_options).await?;
        let db_path = app_dir.join("db.sqlite");
        let conn = Arc::new(Mutex::new(Connection::open(&db_path)?));
        let nb_repository = NotebookRepository::new(conn);
        nb_repository
            .init_db()
            .await
            .expect("Unable it initialize models db");
        info!("Connection to models db established: {:?}", db_path);
        Ok(Notebook {
            embed_store,
            models_store: nb_repository,
        })
    }

    /// Rather than create/update we only have upsert
    /// Update: If id is given
    /// Create: If no id given
    pub async fn upsert_note(
        &mut self,
        id: Option<&str>,
        content: &str,
    ) -> Result<Note, NotebookError> {
        match id {
            // UPDATE
            Some(id) => {
                // get the existing note
                let existing_note = self.models_store.get_note(&id).await?;
                match existing_note {
                    Some(mut note) => {
                        note.set_modified(Self::get_now());
                        self.save_note_text(&note).await?;
                        Ok(note)
                    }
                    None => Err(NotebookError::NoteNotFound(id.to_string())),
                }
            }
            // CREATE
            None => {
                let note = Note::new(&Notebook::generate_id(), &content.to_string());
                info!("Adding new note[{}] to models database", note.get_id());
                self.models_store.add_note(&note).await?;
                info!("Adding new note[{}] to embeddings database", note.get_id());
                self.embed_store
                    .upsert_texts(&[TextChunk {
                        id: note.get_text().to_string(),
                        text: note.get_text().to_string(),
                    }])
                    .await?;
                Ok(note)
            }
        }
    }

    async fn save_note_text(&self, note: &Note) -> Result<(), NotebookError> {
        info!("Saving existing note {} in models db", note.get_id());
        let updated_note = self.models_store.update_note_text(note).await?;
        info!("updating note {} in embeddings db", updated_note.get_id());
        let text_chunk = TextChunk {
            id: updated_note.get_text().to_string(),
            text: updated_note.get_text().to_string(),
        };
        self.embed_store.upsert_texts(&[text_chunk]).await?;
        Ok(())
    }

    pub async fn add_category_to_note(
        &self,
        note_id: &str,
        category_label: &str,
    ) -> Result<Note, NotebookError> {
        let category = self.get_or_create_category(category_label).await?;
        match self.get_note_by_id(note_id).await? {
            Some(mut note) => {
                if !note.has_category(&category) {
                    note.add_category(category);
                    self.models_store.reconcile_note_categories(&note).await?;
                }
                Ok(note.clone())
            }
            None => Err(NotebookError::NoteNotFound(format!(
                "Note: {} not found",
                note_id
            ))),
        }
    }

    pub async fn remove_category_from_note(
        &self,
        note_id: &str,
        category_id: &str,
    ) -> Result<Note, NotebookError> {
        match self.get_note_by_id(note_id).await? {
            Some(mut note) => {
                if let Some(category) = self.get_category_by_id(category_id).await? {
                    if note.has_category(&category) {
                        note.remove_category(category);
                        self.models_store.reconcile_note_categories(&note).await?;
                    }
                    Ok(note.clone())
                } else {
                    // The Cat was not in to models db, ensure the Cat Id is not held within the
                    // Note
                    info!("Category [{}] was not found in the db, ensuring it is not associated with noe: [{}]",category_id, note_id);
                    note.remove_category_by_id(category_id);
                    self.models_store.reconcile_note_categories(&note).await?;
                    Ok(note.clone())
                }
            }
            None => Err(NotebookError::NoteNotFound(format!(
                "Note: {} not found",
                note_id
            ))),
        }
    }

    pub async fn get_or_create_category(&self, cat_label: &str) -> Result<Category, NotebookError> {
        self.models_store.get_or_create_category(cat_label).await
    }

    pub async fn get_category_by_id(
        &self,
        cat_id: &str,
    ) -> Result<Option<Category>, NotebookError> {
        self.models_store.get_category_by_id(cat_id).await
    }

    pub async fn delete_all_notes(&self) -> Result<(), NotebookError> {
        info!("Deleting all notes from models db");
        self.models_store.delete_all_notes().await?;
        info!("Deleting all notes from embed db");
        self.embed_store.empty_db().await?;
        Ok(())
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        self.models_store.get_notes().await
    }

    pub async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>, NotebookError> {
        Ok(self.models_store.get_note(id).await?)
    }

    pub async fn delete_note(&mut self, id: &str) -> Result<(), NotebookError> {
        info!("Deleting note[{}] from models db", id);
        self.models_store.delete_note(id).await?;
        info!("Deleting note[{}] from embeddings db", id);
        self.embed_store.delete_texts(&[id.to_string()]).await?;
        Ok(())
    }

    pub async fn get_note_similars(
        &self,
        note: Note,
        limit: Option<usize>,
        threshold: Option<f32>,
    ) -> Result<Vec<(Note, f32)>, NotebookError> {
        info!("Getting related notes for Note[{}]", note.get_id());
        let limit = limit.unwrap_or(SIMILARS_DEFAULT_LIMIT);
        let threshold = threshold.unwrap_or(SIMILARS_DEFAULT_THRESHOLD);
        let mut result_text_blocks = self
            .embed_store
            .get_similar_to(&note.get_text())
            .limit(limit)
            .threshold(threshold)
            .execute()
            .await
            .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;
        // remove the result_text_blocks where id == note.id
        result_text_blocks.retain(|item| item.id != note.get_id());
        let note_ids = result_text_blocks
            .iter()
            .map(|block| &block.id as &str)
            .collect::<Vec<&str>>();
        let result_notes = self.models_store.get_notes_by_ids(note_ids).await?;
        let notes_map: HashMap<String, Note> = result_notes
            .into_iter()
            .map(|note| (note.get_id().to_string(), note))
            .collect();

        let combined: Vec<(Note, f32)> = result_text_blocks
            .iter()
            .filter_map(|block| {
                notes_map
                    .get(&block.id)
                    .map(|note| (note.clone(), block.distance as f32))
            })
            .collect();
        if combined.is_empty() {
            info!("No similar note found at threshold: {}", threshold)
        }
        Ok(combined)
    }

    pub async fn export_notes(
        &self,
        export_path: PathBuf,
    ) -> Result<(usize, String), NotebookError> {
        if !self.is_writable(&export_path) {
            return Err(NotebookError::FileAccess(format!(
                "{:?} is not writable",
                export_path
            )));
        }

        let export_dir = self.create_export_directory(&export_path)?;
        let notes = self.get_notes().await?;

        for (index, note) in notes.iter().enumerate() {
            let mut note_title = self.note_title(&note);
            let mut note_file_path = export_dir.join(format!("{}.md", note_title));

            if note_file_path.exists() {
                note_title = format!("{}-dupe_{}", note_title, index);
                note_file_path = export_dir.join(format!("{}.md", note_title));
            }

            self.write_note_to_file(&note, &note_file_path)?;
        }

        Ok((notes.len(), export_dir.to_string_lossy().into_owned()))
    }

    pub async fn import_notes(&self, import_path: &Path) -> Result<usize, NotebookError> {
        if !import_path.exists() {
            return Err(NotebookError::FileAccess(format!(
                "{:?} does not exist",
                import_path
            )));
        }

        if !import_path.is_dir() {
            return Err(NotebookError::FileAccess(format!(
                "{:?} is not a directory",
                import_path
            )));
        }

        let mut imported_notes = Vec::new();

        for entry in
        fs::read_dir(&import_path).map_err(|e| NotebookError::FileAccess(e.to_string()))?
        {
            let entry = entry.map_err(|e| NotebookError::FileAccess(e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| NotebookError::FileAccess(e.to_string()))?;
                let note = Note::new(&Notebook::generate_id(), &content);
                // Store the note in the models Db
                self.models_store.add_note(&note).await?;
                imported_notes.push(note);
            }
        }
        // store all the embeddings for the notes
        let text_chunks: Vec<TextChunk> = imported_notes
            .iter()
            .map(|note| note.to_text_chunk())
            .collect();
        self.embed_store.upsert_texts(&text_chunks).await?;

        Ok(imported_notes.len())
    }

    fn note_title(&self, note: &Note) -> String {
        let mut title = note.get_text().lines().next().unwrap_or("").to_string();

        // Strip any leading "#" or spaces from the title
        while title.starts_with('#') || title.starts_with(' ') {
            title.remove(0);
        }

        // Replace any characters not suitable for a file name with "_"
        title = title
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                _ => '_',
            })
            .collect();

        // Ensure the final title string does not have multiple "_"'s in a row
        while title.contains("__") {
            title = title.replace("__", "_");
        }

        // If title is longer than 100 characters, take the first 50 and the last 50, combine them with a "..." in the middle
        if title.len() > 100 {
            let (start, end) = title.split_at(50);
            title = format!("{}...{}", start, end.split_at(end.len() - 50).1);
        }

        title
    }

    fn is_writable(&self, path: &PathBuf) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_dir() {
                if let Ok(permissions) = fs::metadata(path).map(|m| m.permissions()) {
                    return permissions.readonly() == false;
                }
            }
        }
        false
    }

    fn create_export_directory(&self, export_path: &PathBuf) -> Result<PathBuf, NotebookError> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let export_dir_name = format!("knowling_export_{}", timestamp);
        let export_dir = export_path.join(&export_dir_name);
        fs::create_dir_all(&export_dir).map_err(|e| NotebookError::FileAccess(e.to_string()))?;
        Ok(export_dir)
    }

    fn write_note_to_file(&self, note: &Note, file_path: &PathBuf) -> Result<(), NotebookError> {
        let mut file =
            fs::File::create(file_path).map_err(|e| NotebookError::FileAccess(e.to_string()))?;
        file.write_all(note.get_text().as_bytes())
            .map_err(|e| NotebookError::FileAccess(e.to_string()))?;
        Ok(())
    }

    /// Generates a random id for a Document.
    /// The id is a 6-character string composed of alphanumeric characters.
    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
    fn get_now() -> i64 {
        Utc::now().timestamp()
    }
}

#[derive(Error, Debug)]
pub enum NotebookError {
    #[error("Persistence error: {0}")]
    EmbeddingPersistence(#[from] EmbedDbError),

    #[error("Models db error: {0}")]
    ModelPersistence(String),

    #[error("File access error: {0}")]
    FileAccess(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),
}

// rusqlite::Error does not implement Serialize, so we adapt it to a String
impl From<rusqlite::Error> for NotebookError {
    fn from(err: rusqlite::Error) -> NotebookError {
        NotebookError::ModelPersistence(err.to_string())
    }
}

// EmbedDbError does not implement Serialize, so I need to do it manually
impl Serialize for NotebookError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("NotebookError", 2)?;
        match self {
            NotebookError::EmbeddingPersistence(err) => {
                state.serialize_field("type", "EmbeddingPersistence")?;
                state.serialize_field("error", &format!("{:?}", err))?;
            }
            NotebookError::ModelPersistence(err) => {
                state.serialize_field("type", "ModelPersistence")?;
                state.serialize_field("error", err)?;
            }
            NotebookError::FileAccess(err) => {
                state.serialize_field("type", "FileAccess")?;
                state.serialize_field("error", err)?;
            }
            NotebookError::EmbeddingError(err) => {
                state.serialize_field("type", "EmbeddingError")?;
                state.serialize_field("error", err)?;
            }
            NotebookError::NoteNotFound(err) => {
                state.serialize_field("type", "NoteNotFound")?;
                state.serialize_field("error", err)?;
            }
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_notebook_new() {
    //     // todo!("I think i should use dependency injection for the embedding model")
    // }
}

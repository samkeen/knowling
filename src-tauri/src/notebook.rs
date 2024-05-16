use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use fastembed::TextEmbedding;
use log::info;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::notebook::db::{Documentable, EmbedStore, EmbedStoreError};
use crate::notebook::note::Note;
use crate::notebook::notebook_repository::{NotebookRepository, NotebookRepositoryError};

pub mod db;
pub mod note;
mod notebook_repository;

const SIMILARS_DEFAULT_LIMIT: usize = 3;
const SIMILARS_DEFAULT_THRESHOLD: f32 = 0.01;

pub struct Notebook {
    embed_store: EmbedStore,
    models_store: NotebookRepository,
}

impl Notebook {
    pub async fn new(
        text_embedding: TextEmbedding,
        app_dir: &Path,
    ) -> Result<Self, NotebookError> {
        let embed_store = EmbedStore::new(text_embedding, app_dir)
            .await?;
        let db_path = app_dir.join("db.sqlite");
        let conn = Arc::new(Mutex::new(Connection::open(&db_path)?));
        let nb_repository = NotebookRepository::new(conn);
        nb_repository.init_db().await.expect("Unable it initialize models db");
        info!("Connection to models db established: {:?}", db_path);
        Ok(Notebook { embed_store, models_store: nb_repository })
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
                let existing_note: Option<Note> = self.embed_store.get(id).await?;
                match existing_note {
                    Some(mut note) => {
                        note.set_text(content.to_string());
                        info!("updating note {} in models db", note.id());
                        self.models_store.update_note(&note).await?;
                        info!("updating note {} in embeddings db", note.id());
                        self.embed_store.update(vec![note.to_owned()]).await?;
                        Ok(note.clone())
                    }
                    None => Err(NotebookError::NoteNotFound(id.to_string())),
                }
            }
            // CREATE
            None => {
                let mut note = Note::default();
                note.set_id(Notebook::generate_id());
                note.set_text(content.to_string());
                info!("Adding new note[{}] to models database", note.id());
                self.models_store.add_note(&note).await?;
                info!("Adding new note[{}] to embeddings database", note.id());
                self.embed_store.add(vec![note.clone()]).await?;
                info!("Note added to database");
                Ok(note)
            }
        }
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        let (existing_notes, _total_records) = self
            .embed_store
            .get_all()
            .await?;
        Ok(existing_notes)
    }

    pub async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>, NotebookError> {
        self.embed_store
            .get(id)
            .await
            .map_err(|e| NotebookError::EmbeddingPersistence(e))
    }

    pub async fn delete_note(&mut self, id: &str) -> Result<(), NotebookError> {
        info!("Deleting note[{}] from models db", id);
        self.models_store.delete_note(id).await?;
        info!("Deleting note[{}] from embeddings db", id);
        self.embed_store.delete(&vec![id]).await?;
        Ok(())
    }

    pub async fn get_note_similars(
        &self,
        note: Note,
        limit: Option<usize>,
        threshold: Option<f32>,
    ) -> Result<Vec<(Note, f32)>, NotebookError> {
        log::info!("Getting related notes for Note[{}]", note.id());
        let limit = limit.unwrap_or(SIMILARS_DEFAULT_LIMIT);
        let threshold = threshold.unwrap_or(SIMILARS_DEFAULT_THRESHOLD);
        let result = self
            .embed_store
            .search(
                &note.text(),
                Some(format!("id NOT IN ('{}')", note.id()).as_str()),
                // the filter will reduce the number of returned results by 1
                Some(limit + 1),
            )
            .await
            .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;
        let results_outside_threshold: Vec<_> = result
            .clone()
            .into_iter()
            .filter(|i| i.1 > threshold)
            .collect();
        if results_outside_threshold.is_empty() {
            info!("All results met the threshold of {}", threshold);
            return Ok(result);
        }
        info!(
            "{} result(s) did not meet the threshold of {}: {:?}",
            results_outside_threshold.len(),
            threshold,
            results_outside_threshold
                .iter()
                .map(|(note, score)| format!("{}:{}", note.id(), score))
                .collect::<Vec<_>>()
        );
        Ok(result.into_iter().filter(|i| i.1 < threshold).collect())
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
                let mut note = Note::default();
                note.set_id(Notebook::generate_id());
                note.set_text(content);
                imported_notes.push(note);
            }
        }

        self.embed_store
            .add(imported_notes.clone())
            .await?;

        Ok(imported_notes.len())
    }

    fn note_title(&self, note: &Note) -> String {
        let mut title = note.text().lines().next().unwrap_or("").to_string();

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
        file.write_all(note.text().as_bytes())
            .map_err(|e| NotebookError::FileAccess(e.to_string()))?;
        Ok(())
    }

    /// Generates a random id for a Document.
    /// The id is a 6-character string composed of alphanumeric characters.
    pub fn generate_id() -> String {
        let mut rng = rand::thread_rng();
        let id: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(15)
            .collect();
        id
    }
}

#[derive(Error, Debug, Serialize)]
pub enum NotebookError {
    #[error("Persistence error: {0}")]
    EmbeddingPersistence(#[from] EmbedStoreError),

    #[error("Persistence error: {0}")]
    ModelsDbPersistence(#[from] NotebookRepositoryError),

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

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_notebook_new() {
    //     // todo!("I think i should use dependency injection for the embedding model")
    // }
}

use crate::notebook::db::EmbedStore;
use crate::notebook::note::Note;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};

mod db;
pub mod note;

pub struct Notebook {
    notes: Vec<Note>,
    embed_store: EmbedStore,
}

impl Notebook {
    pub async fn new() -> Result<Self, NotebookError> {
        let embedding_model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            show_download_progress: true,
            cache_dir: PathBuf::from(Path::new("../llm-cache")),
            ..Default::default()
        })
        .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;

        let embed_store = EmbedStore::new(embedding_model)
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        let (existing_notes, total_records) = embed_store
            .get_all()
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        Ok(Notebook {
            notes: existing_notes,
            embed_store,
        })
    }

    pub async fn upsert_note(&mut self, id: &str, content: &str) -> Result<Note, NotebookError> {
        if id.is_empty() {
            let note = Note::new(content);
            log::info!("Adding note[{}]", note.get_id());
            self.notes.push(note.clone());
            log::info!("Adding note[{}] to database", note.get_id());
            self.embed_store
                .add(
                    vec![note.get_id().to_string()],
                    vec![note.get_content().to_string()],
                )
                .await
                .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
            log::info!("Note added to database");
            Ok(note)
        } else {
            let note_option = self.notes.iter_mut().find(|note| note.get_id() == id);
            match note_option {
                Some(note) => {
                    note.text = content.to_string();
                    self.embed_store
                        .update(
                            vec![note.get_id().to_string()],
                            vec![note.get_content().to_string()],
                        )
                        .await
                        .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
                    Ok(note.clone())
                }
                None => Err(NotebookError::PersistenceError(format!(
                    "Note with id {} not found",
                    id
                ))),
            }
        }
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        match self.notes.clone() {
            notes => Ok(notes),
        }
    }

    pub fn get_note_by_id(&self, id: &str) -> Option<Note> {
        self.notes.iter().find(|&note| note.get_id() == id).cloned()
    }
}

#[derive(Debug, Serialize)]
pub enum NotebookError {
    PersistenceError(String),
    EmbeddingError(String),
    TableCreationError(String),
}

impl fmt::Display for NotebookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotebookError::PersistenceError(e) => write!(f, "Persistence error: {}", e),
            NotebookError::EmbeddingError(e) => write!(f, "Embedding error: {}", e),
            NotebookError::TableCreationError(e) => write!(f, "Table creation error: {}", e),
        }
    }
}

impl std::error::Error for NotebookError {}

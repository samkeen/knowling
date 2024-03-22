use crate::notebook::db::{EmbedStore, EmbedStoreError};
use crate::notebook::note::Note;
use fastembed::TextEmbedding;
use serde::Serialize;
use std::fmt;

mod db;
pub mod note;

pub struct Notebook {
    notes: Vec<Note>,
    embed_store: EmbedStore,
}

impl Notebook {
    pub async fn new(text_embedding: TextEmbedding) -> Result<Self, NotebookError> {
        let embed_store = EmbedStore::new(text_embedding)
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        let (existing_notes, _total_records) = embed_store
            .get_all()
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        Ok(Notebook {
            notes: existing_notes,
            embed_store,
        })
    }

    pub async fn upsert_note(
        &mut self,
        id: Option<&str>,
        content: &str,
    ) -> Result<Note, NotebookError> {
        match id {
            Some(id) => {
                let note_option = self.notes.iter_mut().find(|note| note.get_id() == id);
                match note_option {
                    Some(note) => {
                        note.text = content.to_string();
                        self.embed_store
                            .update(note.get_id(), note.get_content())
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
            None => {
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

    pub async fn delete_note(&mut self, id: &str) -> Result<(), NotebookError> {
        log::info!("Deleting note[{}]", id);
        self.embed_store
            .delete(&vec![id])
            .await
            .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;
        self.notes.retain(|note| note.get_id() != id);
        Ok(())
    }

    pub async fn get_note_similars(
        &self,
        note: Note,
        limit: Option<usize>,
    ) -> Result<Vec<(Note, f32)>, NotebookError> {
        let limit = limit.unwrap_or(3);
        let result = self
            .embed_store
            .search(&note.text, Some(limit))
            .await
            .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;
        Ok(result)
    }
}

#[derive(Debug, Serialize)]
pub enum NotebookError {
    PersistenceError(String),
    EmbeddingError(String),
    TableCreationError(String),
    NoteNotFound(String),
}

impl fmt::Display for NotebookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotebookError::PersistenceError(e) => write!(f, "Persistence error: {}", e),
            NotebookError::EmbeddingError(e) => write!(f, "Embedding error: {}", e),
            NotebookError::TableCreationError(e) => write!(f, "Table creation error: {}", e),
            NotebookError::NoteNotFound(e) => write!(f, "Table creation error: {}", e),
        }
    }
}

impl std::error::Error for NotebookError {}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_notebook_new() {
    //     // todo!("I think i should use dependency injection for the embedding model")
    // }
}

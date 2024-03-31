use crate::notebook::db::{EmbedStore, EmbedStoreError};
use crate::notebook::note::Note;
use fastembed::TextEmbedding;
use serde::Serialize;
use std::fmt;

mod db;
pub mod note;

const SIMILARS_DEFAULT_LIMIT: usize = 3;
const SIMILARS_DEFAULT_THRESHOLD: f32 = 0.01;

pub struct Notebook {
    embed_store: EmbedStore,
}

impl Notebook {
    pub async fn new(text_embedding: TextEmbedding) -> Result<Self, NotebookError> {
        let embed_store = EmbedStore::new(text_embedding)
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        Ok(Notebook { embed_store })
    }

    pub async fn upsert_note(
        &mut self,
        id: Option<&str>,
        content: &str,
    ) -> Result<Note, NotebookError> {
        match id {
            Some(id) => {
                let existing_note = self
                    .embed_store
                    .get(id)
                    .await
                    .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
                match existing_note {
                    Some(mut note) => {
                        note.text = content.to_string();
                        self.embed_store
                            .update(vec![note.to_owned()])
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
                log::info!("Adding note[{}] to database", note.get_id());
                self.embed_store
                    .add(vec![note.clone()])
                    .await
                    .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
                log::info!("Note added to database");
                Ok(note)
            }
        }
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NotebookError> {
        let (existing_notes, _total_records) = self
            .embed_store
            .get_all()
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))?;
        Ok(existing_notes)
    }

    pub async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>, NotebookError> {
        self.embed_store
            .get(id)
            .await
            .map_err(|e| NotebookError::PersistenceError(e.to_string()))
    }

    pub async fn delete_note(&mut self, id: &str) -> Result<(), NotebookError> {
        log::info!("Deleting note[{}]", id);
        self.embed_store
            .delete(&vec![id])
            .await
            .map_err(|e| NotebookError::EmbeddingError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_note_similars(
        &self,
        note: Note,
        limit: Option<usize>,
        threshold: Option<f32>,
    ) -> Result<Vec<(Note, f32)>, NotebookError> {
        log::info!("Getting related notes for Note[{}]", note.get_id());
        let limit = limit.unwrap_or(SIMILARS_DEFAULT_LIMIT);
        let threshold = threshold.unwrap_or(SIMILARS_DEFAULT_THRESHOLD);
        let result = self
            .embed_store
            .search(
                &note.text,
                Some(format!("id NOT IN ('{}')", note.get_id()).as_str()),
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
            log::info!("All results met the threshold of {}", threshold);
            return Ok(result);
        }
        log::info!(
            "{} result(s) did not meet the threshold of {}: {:?}",
            results_outside_threshold.len(),
            threshold,
            results_outside_threshold
                .iter()
                .map(|(note, score)| format!("{}:{}", note.get_id(), score))
                .collect::<Vec<_>>()
        );
        Ok(result.into_iter().filter(|i| i.1 < threshold).collect())
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

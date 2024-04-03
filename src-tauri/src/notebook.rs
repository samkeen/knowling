use crate::notebook::db::{EmbedStore, EmbedStoreError};
use crate::notebook::note::Note;
use chrono::Utc;
use fastembed::TextEmbedding;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

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

    pub async fn export_notes(&self, export_path: PathBuf) -> Result<usize, NotebookError> {
        // Check if export_path is writable
        if !self.is_writable(&export_path) {
            return Err(NotebookError::FileAccess(format!(
                "{:?} is not writable",
                export_path
            )));
        }

        // Create folder knowling_export at export_path
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let export_dir_name = format!("knowling_export_{}", timestamp);
        let export_dir = export_path.join(&export_dir_name);
        fs::create_dir_all(&export_dir).map_err(|e| NotebookError::FileAccess(e.to_string()))?;

        // Retrieve all notes
        let notes = self.get_notes().await?;

        // For each note write the file to export_path
        for (index, note) in notes.iter().enumerate() {
            let mut note_title = self.note_title(&note);
            let mut note_file_path = export_dir.join(format!("{}.md", note_title));

            // If file of the same name exists, append a suffix
            if note_file_path.exists() {
                note_title = format!("{}-dupe_{}", note_title, index);
                note_file_path = export_dir.join(format!("{}.md", note_title));
            }

            // Write file to {export_path}/knowling_export/{note_title}.md
            let mut file = fs::File::create(note_file_path)
                .map_err(|e| NotebookError::FileAccess(e.to_string()))?;
            file.write_all(note.text.as_bytes())
                .map_err(|e| NotebookError::FileAccess(e.to_string()))?;
        }

        Ok(notes.len())
    }

    fn note_title(&self, note: &Note) -> String {
        let mut title = note.text.lines().next().unwrap_or("").to_string();

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
            title = format!("{}...{}", &title[..50], &title[(title.len() - 50)..]);
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
}

#[derive(Debug, Serialize)]
pub enum NotebookError {
    PersistenceError(String),
    FileAccess(String),
    EmbeddingError(String),
    TableCreationError(String),
    NoteNotFound(String),
}

impl fmt::Display for NotebookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotebookError::PersistenceError(e) => write!(f, "Persistence error: {}", e),
            NotebookError::FileAccess(e) => write!(f, "FileAccess error: {}", e),
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

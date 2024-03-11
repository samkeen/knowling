
use sha2::{Digest, Sha256};
use chrono::Utc;
use serde::Serialize;
use std::fmt::{self, Debug};

#[derive(Debug, Serialize)]
pub enum NoteError {
    DbError(String),
    EmbeddingError(String),
    TableCreationError(String),
}

impl fmt::Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteError::DbError(e) => write!(f, "Database error: {}", e),
            NoteError::EmbeddingError(e) => write!(f, "Embedding error: {}", e),
            NoteError::TableCreationError(e) => write!(f, "Table creation error: {}", e),
        }
    }
}

impl std::error::Error for NoteError {}
#[derive(Debug, Clone, Serialize)]
pub struct Note {
    id: String,
    content: String
}

impl Note {
    pub fn new(content: &str) -> Self {
        Note {
            id: Self::generate_id(),
            content: content.to_string()
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    fn generate_id() -> String {
        let mut hasher = Sha256::new();
        hasher.update(Utc::now().to_string());
        hex::encode(hasher.finalize())[..6].to_string()
    }


}



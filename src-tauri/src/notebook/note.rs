use serde::Serialize;
use vec_embed_store::TextChunk;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    id: String,
    text: String,
    created: i64,
    modified: i64,
}

impl Note {
    pub(crate) fn new(id: &str, text: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self::hydrate(id, text, now, now)
    }
    pub(crate) fn hydrate(id: &str, text: &str, created: i64, modified: i64) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            created,
            modified,
        }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }
    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }
    pub(crate) fn get_created(&self) -> i64 {
        self.created
    }
    pub(crate) fn get_modified(&self) -> i64 {
        self.modified
    }

    pub(crate) fn set_modified(&mut self, timestamp: i64) {
        self.modified = timestamp;
    }

    pub(crate) fn to_text_chunk(&self) -> TextChunk {
        TextChunk { id: self.id.to_string(), text: self.text.to_string() }
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        let note = Note::new("1", "Test Note");
        assert_eq!("1", note.get_id());
    }

    #[test]
    fn test_get_content() {
        let content = "Test Note";
        let mut note = Note::new("1", "foo");
        note.set_text(content);
        assert_eq!(note.get_text(), content);
    }
}

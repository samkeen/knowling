use serde::Serialize;

use crate::notebook::db::Documentable;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Note {
    id: String,
    text: String,
    created: i64,
    modified: i64,
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Documentable for Note {
    fn id(&self) -> &str {
        &self.id
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn created(&self) -> i64 {
        self.created
    }

    fn modified(&self) -> i64 {
        self.modified
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn set_created(&mut self, created: i64) {
        self.created = created;
    }

    fn set_modified(&mut self, modified: i64) {
        self.modified = modified;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        let mut note = Note::default();
        note.set_text("Test Note".to_string());
        assert!(!note.id().is_empty());
    }

    #[test]
    fn test_get_content() {
        let content = "Test Note";
        let mut note = Note::default();
        note.set_text(content.to_string());
        assert_eq!(note.text(), content);
    }
}

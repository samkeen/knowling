use crate::notebook::db::Document;

/// Alias the Document type to Note
pub type Note = Document;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        let note = Note::new("Test Note");
        assert!(!note.get_id().is_empty());
    }

    #[test]
    fn test_get_content() {
        let content = "Test Note";
        let note = Note::new(content);
        assert_eq!(note.get_content(), content);
    }
}

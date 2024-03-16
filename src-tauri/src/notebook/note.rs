use crate::notebook::db::Document;
use rand::distributions::Alphanumeric;
use rand::Rng;

/// Alias and extend the Document type
pub type Note = Document;

// Implement additional methods for the Note type.
impl Note {
    /// Creates a new Note with the given content.
    /// The Note's id is generated randomly.
    pub fn new(content: &str) -> Self {
        Note {
            id: Self::generate_id(),
            text: content.to_string(),
        }
    }

    /// Returns the id of the Note.
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the content of the Note.
    pub fn get_content(&self) -> &str {
        &self.text
    }

    /// Generates a random id for a Note.
    /// The id is a 6-character string composed of alphanumeric characters.
    fn generate_id() -> String {
        let mut rng = rand::thread_rng();
        let id: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(6)
            .collect();
        id
    }
}

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

    #[test]
    fn test_generated_id_length() {
        assert_eq!(Note::generate_id().len(), 6);
    }

    #[test]
    fn test_generated_id_unique() {
        assert_ne!(Note::generate_id(), Note::generate_id());
    }
}

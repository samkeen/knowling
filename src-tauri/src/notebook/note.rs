use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vec_embed_store::TextChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    id: String,
    label: String,
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Category {}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Category {
    pub(crate) fn new(label: &str) -> Self {
        Self::hydrate(&Self::generate_id(), label)
    }
    pub(crate) fn hydrate(id: &str, label: &str) -> Self {
        Category {
            id: id.to_string(),
            label: label.to_string(),
        }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }
    pub(crate) fn get_label(&self) -> &str {
        &self.label
    }
    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    id: String,
    text: String,
    categories: HashSet<Category>,
    created: i64,
    modified: i64,
}

impl Note {}

impl Note {
    pub(crate) fn new(id: &str, text: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self::hydrate(id, text, HashSet::new(), now, now)
    }

    pub(crate) fn hydrate(
        id: &str,
        text: &str,
        categories: HashSet<Category>,
        created: i64,
        modified: i64,
    ) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            categories,
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
    pub(crate) fn get_categories(&self) -> &HashSet<Category> {
        &self.categories
    }

    pub(crate) fn get_created(&self) -> i64 {
        self.created
    }
    pub(crate) fn get_modified(&self) -> i64 {
        self.modified
    }

    pub(crate) fn has_category(&self, category: &Category) -> bool {
        self.categories.contains(category)
    }

    pub(crate) fn add_category(&mut self, category: Category) {
        self.categories.insert(category);
    }

    pub(crate) fn remove_category(&mut self, category: Category) {
        self.categories.remove(&category);
    }

    pub(crate) fn remove_category_by_id(&mut self, category_id: &str) {
        self.categories.retain(|c| c.id != category_id);
    }


    pub(crate) fn set_modified(&mut self, timestamp: i64) {
        self.modified = timestamp;
    }

    pub(crate) fn to_text_chunk(&self) -> TextChunk {
        TextChunk {
            id: self.id.to_string(),
            text: self.text.to_string(),
        }
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
        let mut note = Note::new("1", content);
        assert_eq!(note.get_text(), content);
    }
}

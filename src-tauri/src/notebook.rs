use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use crate::notebook::db::EmbedStore;
use crate::notebook::note::{Note, NoteError};

mod db;
pub mod note;

pub struct Notebook {
    notes: Vec<Note>,
    embed_store: EmbedStore
}

impl Notebook {
    pub async fn new() -> Self {
        let embedding_model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            show_download_progress: true,
            ..Default::default()
        }).unwrap();
        let embed_store = EmbedStore::new(embedding_model).await.unwrap();
        Notebook { notes: Vec::new(), embed_store }
    }

    pub async fn add_note(&mut self, content: &str) -> Result<Note, NoteError> {
        let note = Note::new(content);
        log::info!("Adding note[{}]", note.get_id());
        self.notes.push(note.clone());
        log::info!("Adding note to database");
        log::info!("embeddings created");
        self.embed_store.add(vec![note.get_content().to_string()],
                             vec![note.get_id().to_string()]).await.unwrap();
        log::info!("Note added to database");
        Ok(note)
    }

    pub async fn get_notes(&self) -> Result<Vec<Note>, NoteError> {
        match self.notes.clone() {
            notes => Ok(notes),
        }
    }

    pub fn get_note_by_id(&self, id: &str) -> Option<Note> {
        self.notes.iter().find(|&note| note.get_id() == id).cloned()
    }

}

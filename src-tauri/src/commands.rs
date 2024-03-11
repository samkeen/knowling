use crate::notebook::Notebook;
use crate::notebook::note::{Note, NoteError};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub async fn save_note(text: &str) -> Result<String, String> {
    log::info!("Saving note: '{}'", text);
    let mut notebook = Notebook::new().await;
    match notebook.add_note(text).await {
        Ok(note) => Ok(format!("Note[{}] recorded", note.get_id())),
        Err(e) => Err(format!("Error: {}", e))
    }
}

#[tauri::command]
pub async fn get_notes() -> Result<Vec<Note>, NoteError> {
    let notebook = Notebook::new().await;
    let notes = notebook.get_notes().await?;
    log::info!("Found [{}] existing notes", notes.len());
    Ok(notes)
}



use crate::notebook::{Notebook, NotebookError};
use crate::notebook::note::Note;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub async fn save_note(text: &str) -> Result<String, String> {
    log::info!("Saving note: '{}'", text);
    let mut notebook = match Notebook::new().await {
        Ok(notebook) => notebook,
        Err(e) => return Err(format!("Error: {}", e))
    };
    match notebook.add_note(text).await {
        Ok(note) => Ok(format!("Note[{}] recorded", note.get_id())),
        Err(e) => Err(format!("Error: {}", e))
    }
}

#[tauri::command]
pub async fn get_notes() -> Result<Vec<Note>, NotebookError> {
    let notebook = match Notebook::new().await {
        Ok(notebook) => notebook,
        Err(e) => return Err(e)
    };
    let notes = notebook.get_notes().await?;
    log::info!("Found [{}] existing notes", notes.len());
    Ok(notes)
}



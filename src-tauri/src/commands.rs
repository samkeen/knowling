use tauri::State;
use crate::AppState;
use crate::notebook::{NotebookError};
use crate::notebook::note::Note;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub async fn save_note(notebook: State<'_, AppState>, text: &str) -> Result<String, String> {
    log::info!("Saving note: '{}'", text);
    let mut notebook = notebook.notebook.lock().await;
    match notebook.add_note(text).await {
        Ok(note) => Ok(format!("Note[{}] recorded", note.get_id())),
        Err(e) => Err(format!("Error: {}", e))
    }
}

#[tauri::command]
pub async fn get_notes(notebook: State<'_, AppState>) -> Result<Vec<Note>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let notes = notebook.get_notes().await?;
    log::info!("Found [{}] existing notes", notes.len());
    Ok(notes)
}

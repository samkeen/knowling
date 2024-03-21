use crate::notebook::note::Note;
use crate::notebook::NotebookError;
use crate::AppState;
use tauri::State;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub async fn save_note(
    notebook: State<'_, AppState>,
    id: Option<&str>,
    text: &str,
) -> Result<String, String> {
    log::info!("Saving note: '{}'", text);
    let mut notebook = notebook.notebook.lock().await;
    match notebook.upsert_note(id, text).await {
        Ok(note) => Ok(format!("Note[{}] recorded", note.get_id())),
        Err(e) => Err(format!("Error: {}", e)),
    }
}

#[tauri::command]
pub async fn get_notes(notebook: State<'_, AppState>) -> Result<Vec<Note>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let notes = notebook.get_notes().await?;
    log::info!("Found [{}] existing notes", notes.len());
    Ok(notes)
}

#[tauri::command]
pub async fn get_note_by_id(
    notebook: State<'_, AppState>,
    id: &str,
) -> Result<Note, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let note = notebook.get_note_by_id(id);
    note.ok_or(NotebookError::NoteNotFound(format!(
        "No note found with id: {}",
        id
    )))
}

#[tauri::command]
pub async fn get_note_similarities(
    notebook: State<'_, AppState>,
    id: &str,
) -> Result<Vec<(Note, f32)>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let note = notebook.get_note_by_id(id);
    match note {
        Some(note) => notebook.get_note_similars(note, Some(3)).await,
        None => Err(NotebookError::NoteNotFound(format!(
            "No note found with id: {}",
            id
        ))),
    }
}

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
) -> Result<Note, String> {
    log::info!("Saving note: '{}'", text);
    let mut notebook = notebook.notebook.lock().await;
    match notebook.upsert_note(id, text).await {
        Ok(note) => {
            log::info!("Note[{}] saved", note.id);
            Ok(note)
        }
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
pub async fn export_notes(
    notebook: State<'_, AppState>,
    export_path: &str,
) -> Result<usize, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let num_exported = notebook.export_notes(export_path).await?;
    log::info!("Exported [{}] existing notes", num_exported);
    Ok(num_exported)
}

#[tauri::command]
pub async fn get_note_by_id(
    notebook: State<'_, AppState>,
    id: &str,
) -> Result<Option<Note>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    match notebook.get_note_by_id(id).await {
        Ok(note) => Ok(note),
        Err(e) => {
            log::error!("Error getting note by id: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn delete_note(notebook: State<'_, AppState>, id: &str) -> Result<(), NotebookError> {
    let mut notebook = notebook.notebook.lock().await;
    notebook.delete_note(id).await
}

#[tauri::command]
pub async fn get_note_similarities(
    notebook: State<'_, AppState>,
    id: &str,
) -> Result<Vec<(Note, f32)>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let note = notebook.get_note_by_id(id).await?;
    match note {
        Some(note) => notebook.get_note_similars(note, Some(3), Some(0.5)).await,
        None => Err(NotebookError::NoteNotFound(format!(
            "No note found with id: {}",
            id
        ))),
    }
}

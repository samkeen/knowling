use crate::notebook::note::Note;
use crate::notebook::NotebookError;
use crate::AppState;
use std::path::PathBuf;
use tauri::api::path::download_dir;
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
pub async fn export_notes(notebook: State<'_, AppState>) -> Result<(usize, String), NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let target_dir = download_dir().ok_or(NotebookError::FileAccess(
        "Failed to resolve path to downloads directory".to_string(),
    ))?;
    let export_result = notebook.export_notes(target_dir.clone()).await?;
    log::info!(
        "Exported [{}] existing notes to {}",
        export_result.0,
        export_result.1
    );
    Ok(export_result)
}

#[tauri::command]
pub async fn import_notes(
    notebook: State<'_, AppState>,
    path: &str,
) -> Result<usize, NotebookError> {
    log::info!("Attempting import of notes from: {}", path);
    let notebook = notebook.notebook.lock().await;
    let num_imports = notebook.import_notes(&PathBuf::from(path)).await?;
    log::info!("Imported [{}] notes from {}", num_imports, path);
    Ok(num_imports)
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

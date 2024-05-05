use std::path::PathBuf;

use log::info;
use serde_json::json;
use tauri::api::path::download_dir;
use tauri::State;
use tauri_plugin_store::StoreBuilder;

use crate::AppState;
use crate::llm::llm_request;
use crate::notebook::note::Note;
use crate::notebook::NotebookError;

const SYS_PROMPT_CONSIDER_NOTE: &str = r#"You are a personal assistant. You advise on notes
 presented to you. Notes presented to you are created by the user.  In your answers to strive
 to improve understanding and clarity of the note for the user. Format all responses in valid Markdown
 but do not surround the response in ticks (```)"#;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub async fn save_note(
    notebook: State<'_, AppState>,
    id: Option<&str>,
    text: &str,
) -> Result<Note, String> {
    info!("Saving note: '{}'", text);
    let mut notebook = notebook.notebook.lock().await;
    match notebook.upsert_note(id, text).await {
        Ok(note) => {
            info!("Note[{}] saved", note.id);
            Ok(note)
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

// #[tauri::command]
// pub async fn add_category_to_note(
//     notebook: State<'_, AppState>,
//     category: &str,
// ) -> Result<Note, String> {
//     info!("Adding category: '{}' to note [{}]", text);
//     let mut notebook = notebook.notebook.lock().await;
//     match notebook.upsert_note(id, text).await {
//         Ok(note) => {
//             info!("Note[{}] saved", note.id);
//             Ok(note)
//         }
//         Err(e) => Err(format!("Error: {}", e)),
//     }
// }

#[tauri::command]
pub async fn prompt_about_note(notebook: State<'_, AppState>, app_handle: tauri::AppHandle,
                               prompt: &str, note_id: &str) -> Result<String, String> {
    info!("running command prompt_about_note");
    let notebook = notebook.notebook.lock().await;
    match notebook.get_note_by_id(note_id).await {
        Ok(note) => {
            let mut store = StoreBuilder::new(app_handle, PathBuf::from("settings.json")).build();
            let _ = store.load();
            let default_value = json!("");
            let api_key = match store.get("anthropicApiKey")
                .unwrap_or(&default_value)
                .as_str()
                .map(|key| key.to_string()) {
                Some(key) => key,
                None => return Err("Unable to retrieve LLM API Key".into()),
            };
            let note_content = match note
            {
                None => { "".to_string() }
                Some(note) => { note.text }
            };
            if api_key.is_empty() { return Err("Unable to retrieve LLM API Key".to_string()); }
            // combine the prompt and the note content
            let result = llm_request(
                &format!("{}\n<note-content>{}\n</note-content>", prompt, note_content),
                &api_key,
                Some(SYS_PROMPT_CONSIDER_NOTE)).await;
            match result {
                Ok(result) => {
                    Ok(result.first_message())
                }
                Err(err) => {
                    Err(format!("Error: {}", err))
                }
            }
        }
        Err(err) => {
            Err(format!("Note with id: {note_id} not found. ({err})").to_string())
        }
    }
}


#[tauri::command]
pub async fn get_notes(notebook: State<'_, AppState>) -> Result<Vec<Note>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let notes = notebook.get_notes().await?;
    info!("Found [{}] existing notes", notes.len());
    Ok(notes)
}

#[tauri::command]
pub async fn export_notes(notebook: State<'_, AppState>) -> Result<(usize, String), NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let target_dir = download_dir().ok_or(NotebookError::FileAccess(
        "Failed to resolve path to downloads directory".to_string(),
    ))?;
    let export_result = notebook.export_notes(target_dir.clone()).await?;
    info!(
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
    info!("Attempting import of notes from: {}", path);
    let notebook = notebook.notebook.lock().await;
    let num_imports = notebook.import_notes(&PathBuf::from(path)).await?;
    info!("Imported [{}] notes from {}", num_imports, path);
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
    threshold: f32,
) -> Result<Vec<(Note, f32)>, NotebookError> {
    let notebook = notebook.notebook.lock().await;
    let note = notebook.get_note_by_id(id).await?;
    match note {
        Some(note) => {
            notebook
                .get_note_similars(note, Some(3), Some(threshold))
                .await
        }
        None => Err(NotebookError::NoteNotFound(format!(
            "No note found with id: {}",
            id
        ))),
    }
}

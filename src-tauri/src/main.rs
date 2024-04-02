// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod notebook;

use crate::commands::get_note_similarities;
use crate::notebook::Notebook;
use commands::{delete_note, export_notes, get_note_by_id, get_notes, save_note};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    // See https://github.com/tauri-apps/tauri/discussions/1336#discussioncomment-1936523
    pub notebook: Arc<Mutex<Notebook>>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    env_logger::init();
    let text_embedding = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2,
        show_download_progress: true,
        cache_dir: PathBuf::from(Path::new("../llm-cache")),
        ..Default::default()
    })
    .unwrap();
    // block until we get the Notebook
    let app_state = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let notebook = Arc::new(Mutex::new(Notebook::new(text_embedding).await.unwrap()));
            AppState { notebook }
        });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            save_note,
            get_notes,
            export_notes,
            get_note_by_id,
            get_note_similarities,
            delete_note,
        ])
        // @TODO see https://blog.moonguard.dev/how-to-use-local-sqlite-database-with-tauri
        // .setup(|_app| {
        //     // Initialize the database.
        //     db::init();
        //
        //     Ok(())
        // })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod notebook;
mod utils;

use crate::commands::get_note_similarities;
use crate::notebook::Notebook;
use crate::utils::{get_user_app_dir, set_panic_hook};
use commands::{delete_note, export_notes, get_note_by_id, get_notes, import_notes, save_note};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use log::LevelFilter;
use std::io::Write;
use std::sync::Arc;
use tauri_plugin_log::LogTarget;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    // See https://github.com/tauri-apps/tauri/discussions/1336#discussioncomment-1936523
    pub notebook: Arc<Mutex<Notebook>>,
}
// adapt log tagets based on prod/non-prod
#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::Webview];
#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    let app_dir = get_user_app_dir();
    set_panic_hook(&app_dir);

    let text_embedding = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2,
        show_download_progress: true,
        cache_dir: app_dir.join("llm-cache"),
        ..Default::default()
    })
    .unwrap();
    // block until we get the Notebook
    let app_state = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let notebook = Arc::new(Mutex::new(
                Notebook::new(text_embedding, &app_dir).await.unwrap(),
            ));
            AppState { notebook }
        });

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(LOG_TARGETS)
                .level(LevelFilter::Info)
                .build(),
        )
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            save_note,
            get_notes,
            export_notes,
            import_notes,
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

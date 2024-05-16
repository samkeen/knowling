// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use log::LevelFilter;
use tauri::Manager;
use tauri_plugin_log::LogTarget;
use tokio::sync::Mutex;

use commands::{add_category_to_note, delete_all_notes, delete_note, export_notes, get_note_by_id,
               get_notes, import_notes, prompt_about_note, save_note};

use crate::commands::get_note_similarities;
use crate::notebook::Notebook;
use crate::utils::{get_user_app_dir, set_panic_hook};

mod commands;
mod notebook;
mod utils;
mod llm;

#[derive(Clone)]
pub struct AppState {
    // See https://github.com/tauri-apps/tauri/discussions/1336#discussioncomment-1936523
    pub notebook: Arc<Mutex<Notebook>>,
}

// adapt log targets based on prod/non-prod
#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::Webview];
#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    let app_dir = get_user_app_dir();
    // log here in the event of a panic
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
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(LOG_TARGETS)
                .level(LevelFilter::Warn)
                .level_for("knowling", LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_store::Builder::default().build()
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
            prompt_about_note,
            add_category_to_note,
            delete_all_notes
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

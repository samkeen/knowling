// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use log::LevelFilter;
use tauri::Manager;
use tauri_plugin_log::LogTarget;
use tokio::sync::Mutex;
use vec_embed_store::EmbeddingEngineOptions;

use commands::{add_category_to_note, delete_all_notes, delete_note, export_notes, get_note_by_id,
               get_notes, import_notes, prompt_about_note, remove_category_from_note, save_note_text};

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

    let embedding_engine_options = EmbeddingEngineOptions {
        show_download_progress: true,
        cache_dir: app_dir.join("llm-cache"),
        ..Default::default()
    };
    // block until we get the Notebook
    let app_state = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let notebook = Arc::new(Mutex::new(
                Notebook::new(embedding_engine_options, app_dir.as_path()).await.unwrap(),
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
                .level(LevelFilter::Info)
                // .level_for("knowling", LevelFilter::Info) // this does not seem to work???
                .build(),
        )
        .plugin(
            tauri_plugin_store::Builder::default().build()
        )
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            save_note_text,
            get_notes,
            export_notes,
            import_notes,
            get_note_by_id,
            get_note_similarities,
            delete_note,
            prompt_about_note,
            add_category_to_note,
            delete_all_notes,
            remove_category_from_note
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

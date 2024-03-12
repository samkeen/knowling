// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod notebook;
mod commands;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::notebook::Notebook;
use commands::save_note;
use commands::get_notes;

#[derive(Clone)]
pub struct AppState {
    pub notebook: Arc<Mutex<Notebook>>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    env_logger::init();
    // block until we get the Notebook 
    let app_state = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let notebook = Arc::new(Mutex::new(Notebook::new().await.unwrap()));
            AppState { notebook }
        });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            save_note,
            get_notes
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

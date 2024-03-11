// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod notebook;
mod commands;

use commands::save_note;
use commands::get_notes;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
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

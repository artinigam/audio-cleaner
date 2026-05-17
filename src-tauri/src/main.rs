// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod enhancement;
mod ffmpeg;
mod models;
mod utils;

use commands::media::{enhance_audio_file, extract_audio_from_media, probe_media_file};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            probe_media_file,
            extract_audio_from_media,
            enhance_audio_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

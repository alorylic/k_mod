// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path, sync::Mutex};

use service::*;

mod entry;
mod service;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub struct GamePath(Mutex<Option<String>>);

fn main() {
    if !path::Path::new("./game_setting.json").exists() {
        fs::File::create("./game_setting.json").unwrap();
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .open("./game_setting.json")
        .unwrap();
    let game_path: Option<String> = serde_json::from_reader(file).ok();
    
    tauri::Builder::default()
        .manage(GamePath(Mutex::new(game_path)))
        .invoke_handler(
            tauri::generate_handler![
                greet,
                get_mods,
                add_mod,
                use_the_mod,
                unuse_mod,
                remove_mod,
                clear_game_file_backup,
                set_game_path,
                get_game_path
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

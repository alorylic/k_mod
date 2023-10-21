#![allow(unused)]
mod file_deal;
mod db_deal;

use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry, ReadDir},
    path::{self, Path, PathBuf}, io::Write, cmp::Reverse
};

use log::info;
use rusqlite::Connection;
use serde_json::error;
use tauri::State;

use crate::{
    entry::{ModInfo},GamePath,
};

static MODFILE: &str = "mods";
static GAMEFILE: &str = "_game_file";

#[tauri::command]
pub fn get_mods() -> Vec<ModInfo> {
    let mut mods = db_deal::get_mods_from_db();
    mods.sort_by_key(|it| Reverse(it.insert_time.clone()));
    mods
}

/// 添加mod
///
#[tauri::command]
pub fn add_mod(path: String) -> Result<ModInfo, String> {
    let mod_info = file_deal::get_mod_from_dir(path.clone());
    match mod_info {
        Some(mut mod_info) => {
            if !db_deal::check_mod_exist(&mod_info.name) {
                return Err("mod已存在".into());
            }
            file_deal::add_mod_file(&path, &mut mod_info);
            db_deal::add_mod_to_db(&mod_info)?;
            Ok(mod_info)
        }
        None => Err("mod文件夹不存在".into()),
    }
}


/// 启用mod
///
#[tauri::command]
pub fn use_the_mod(mut mod_info: ModInfo, game_path: State<GamePath>) {
    let game_path = game_path.0.lock().unwrap().as_ref().unwrap().clone();
    // 获取mod文件地址 -> 游戏读取mod文件地址映射
    let pattern = format!(".\\{}\\{}", MODFILE, mod_info.name);
    let mod_pattern = format!(".\\{}\\{}\\", MODFILE, mod_info.name);
    let target = game_path.clone();
    let get_target_path = move |it: &String| it.replace(&pattern, &target);

    // 获取原始文件信息
    let mut game_file_db = db_deal::get_game_file_from_db();
    let mut more_file_path = HashSet::new();
    for path in mod_info.path.as_ref().unwrap() {
        let target_path = get_target_path(path);
        // 备份会被覆盖的原始文件
        if !game_file_db.contains(&target_path) && Path::new(&target_path).exists() {
            more_file_path.insert(target_path.clone());
            let game_mode_file =
                path.replace(&mod_pattern, &format!(".\\{}\\{}\\", MODFILE, GAMEFILE));
            file_deal::make_dir(&game_mode_file);
            fs::copy(&target_path, &game_mode_file).unwrap();
        }
        file_deal::make_dir(&target_path);
        let n = Path::new(&path).exists();
        info!("{}", n);
        fs::copy(&path, &target_path).unwrap();
    }
    // 更新原始文件信息
    game_file_db.extend(more_file_path.into_iter());
    db_deal::add_game_file_to_db(&game_file_db);

    mod_info.apply = true;
    db_deal::update_mod_to_db(&mod_info)
}

/// 移除mod
///
#[tauri::command]
pub fn remove_mod(mod_info: ModInfo) {
    // 删除MODFILE中的mod目录
    let path = format!(".\\{}\\{}", MODFILE, mod_info.name);
    fs::remove_dir_all(&path).unwrap();
    // 删除数据库中的mod信息
    let conn = Connection::open("./mod_manager.db").unwrap();
    let mut stmt = conn
        .prepare("delete from mod_info where name = ?1")
        .unwrap();
    let name = mod_info.name.clone();
    stmt.execute([&name]);
    stmt = conn
        .prepare("delete from mod_file_info where name = ?1")
        .unwrap();
    stmt.execute([&name]);
}

/// 关闭mod
///
#[tauri::command]
pub fn unuse_mod(mut mod_info: ModInfo, game_path: State<GamePath>) {
    let game_path = game_path.0.lock().unwrap().as_ref().unwrap().clone();

    let pattern = format!(".\\{}\\{}", MODFILE, mod_info.name);
    let target = game_path.clone();

    let get_game_path = move |it: &String| it.replace(&pattern, &target);
    let get_raw_path = move |it: &String| {
        it.replace(
            &game_path.clone(),
            &format!(".\\{}\\{}", MODFILE, GAMEFILE),
        )
    };

    let mut game_file_db = db_deal::get_game_file_from_db();

    for path in mod_info.path.as_ref().unwrap() {
        // 删除启用的mod文件
        let game_path = get_game_path(path);
        fs::remove_file(&game_path);
        // 恢复被覆盖的原始游戏文件
        if game_file_db.contains(&game_path) {
            let row_mod_path = get_raw_path(&game_path);
            fs::copy(&row_mod_path, &game_path);
            game_file_db.remove(&game_path);
        }
    }
    // 更新原始文件信息
    mod_info.apply = false;
    db_deal::update_mod_to_db(&mod_info);
}

#[tauri::command]
pub fn clear_game_file_backup() {
    db_deal::remove_all_game_file();
    file_deal::remove_all_game_file();
}

#[tauri::command]
pub fn set_game_path(path: String, mut game_path: State<GamePath>) {
    let mut game_path = game_path.0.lock();
    game_path.as_mut().unwrap().replace(path);
    let path = game_path.as_ref().unwrap().as_ref().unwrap();
    fs::File::create("./game_setting.json")
        .unwrap()
        .write_all(serde_json::to_string(path).unwrap().as_bytes())
        .unwrap();
}

#[tauri::command]
pub fn get_game_path(mut game_path: State<GamePath>) -> Result<String, String> {
    game_path.0.lock().unwrap().clone().ok_or("游戏路径未设置".into())
}
use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry, ReadDir},
    path::{self, Path, PathBuf}
};

use log::info;
use rusqlite::Connection;

use crate::entry::{self, ModFileInfo, ModInfo};

/// 获取数据库中的mod信息
///
pub fn get_mods_from_db() -> Vec<ModInfo> {
    let conn = Connection::open("./mod_manager.db").unwrap();
    // 获取mod文件数据
    let mut stmt = conn
        .prepare("select name, path from mod_file_info where name != '_game_file'")
        .unwrap();
    let mod_files = stmt
        .query_map([], |row| {
            Ok(ModFileInfo {
                name: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
            })
        })
        .unwrap();
    let mod_files: Vec<_> = mod_files.into_iter().map(|it| it.unwrap()).collect();

    // 获取mod主体数据
    let mut stmt = conn
        .prepare("select name, create_time, apply from mod_info")
        .unwrap();
    let mod_list = stmt
        .query_map([], |row| {
            let x = row.get::<_, String>(1).unwrap();
            let apply = if row.get::<_, i32>(2).unwrap() == 1 {
                true
            } else {
                false
            };
            Ok(ModInfo {
                name: row.get(0).unwrap(),
                insert_time: serde_json::from_str(&x).unwrap(),
                apply,
                path: None,
            })
        })
        .unwrap();

    // 组合mod数据
    let mut name_to_mod: HashMap<String, ModInfo> = mod_list
        .into_iter()
        .map(|it| {
            let it = it.unwrap();
            (it.name.clone(), it)
        })
        .collect();
    for file in mod_files {
        name_to_mod
            .get_mut(&file.name)
            .unwrap()
            .path
            .get_or_insert(vec![])
            .push(file.path.clone());
    }

    name_to_mod.into_iter().map(|it| it.1).collect()
}

/// 新增mod信息到数据库中
///
pub fn add_mod_to_db(mod_info: &ModInfo) -> Result<(), String> {
    let conn = Connection::open("./mod_manager.db").unwrap();
    // 判断是否存在重名mod
    let name = mod_info.name.clone();
    let mut stmt = conn
        .prepare("select count(*) from mod_info where name = ?1")
        .unwrap();
    let mut rows = stmt.query([name]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        if row.get::<_, i32>(0).unwrap() >= 1 {
            return Err("已存在同名mod".into());
        }
    }

    // 新增mod主体数据
    let mut stmt = conn
        .prepare("insert into mod_file_info (name, path) values (?1, ?2)")
        .unwrap();
    for path in mod_info.path.as_ref().unwrap() {
        let param = (mod_info.name.clone(), path.clone());
        stmt.execute(param).unwrap();
    }

    // 新增mod文件数据
    stmt = conn
        .prepare("insert into mod_info (name, apply, create_time) values (?1, ?2,?3)")
        .unwrap();
    let param = (
        mod_info.name.clone(),
        "0",
        serde_json::to_string(&mod_info.insert_time).unwrap(),
    );
    stmt.execute(param).unwrap();

    Ok(())
}

/// 判断当前mod是否已经存在
///
pub fn check_mod_exist(name: &String) -> bool {
    let conn = Connection::open("./mod_manager.db").unwrap();
    let mut stmt = conn
        .prepare("select count(*) from mod_info where name = ?1")
        .unwrap();
    let mut rows = stmt.query([name]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        if row.get::<_, i32>(0).unwrap() >= 1 {
            return false;
        }
    }
    return true;
}

/// 判断当前文件是否已经被已有mod占用，暂未启用
///
pub fn exist_mod_path(path: &String) -> Vec<String> {
    let conn = Connection::open("./mod_manager.db").unwrap();
    let mut stmt = conn
        .prepare("select name from mod_info where path = ?1")
        .unwrap();
    let mut rows = stmt
        .query_map([path], |row| Ok(row.get::<_, String>(0).unwrap()))
        .unwrap();
    rows.into_iter().map(|it| it.unwrap()).collect()
}

/// 更新mod启用信息
///
pub fn update_mod_to_db(mod_info: &ModInfo) {
    let conn = Connection::open("./mod_manager.db").unwrap();
    let name = mod_info.name.clone();
    let mut stmt = conn
        .prepare("update mod_info set apply = ?1 where name = ?2")
        .unwrap();
    let apply = if mod_info.apply { 1 } else { 0 };
    let params = (apply, name);
    stmt.execute(params).unwrap();
}

/// 获取原始游戏文件备份信息
///
pub fn get_game_file_from_db() -> HashSet<String> {
    let conn = Connection::open("./mod_manager.db").unwrap();
    let mut stmt = conn
        .prepare("select path from mod_file_info where name = '_game_file'")
        .unwrap();
    let mod_file_list = stmt
        .query_map([], |row| Ok(row.get::<_, String>(0).unwrap()))
        .unwrap();
    mod_file_list.into_iter().map(|it| it.unwrap()).collect()
}

/// 添加原始游戏文件备份信息
///
pub fn add_game_file_to_db(paths: &HashSet<String>) {
    let conn = Connection::open("./mod_manager.db").unwrap();
    // 删除原数据
    let mut stmt = conn
        .prepare("delete from mod_file_info where name = '_game_file'")
        .unwrap();
    stmt.execute([]).unwrap();
    //插入新数据
    stmt = conn
        .prepare("insert into mod_file_info (name, path) values (?1, ?2)")
        .unwrap();
    for path in paths {
        let params = ("_game_file", path);
        stmt.execute(params).unwrap();
    }
}

pub fn remove_all_game_file() {
    let conn = Connection::open("./mod_manager.db").unwrap();
    // 删除原数据
    let mut stmt = conn
        .prepare("delete from mod_file_info where name = '_game_file'")
        .unwrap();
    stmt.execute([]).unwrap();
}
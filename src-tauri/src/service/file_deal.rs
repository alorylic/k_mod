/// 处理文件相关逻辑
/// 

use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry, ReadDir},
    path::{self, Path, PathBuf}
};

use log::info;
use rusqlite::Connection;

use crate::{
    entry::{self, ModFileInfo, ModInfo},
    service::MODFILE
};

use super::GAMEFILE;
/// 读取指定路径下的所有mod文件，仅读取文件，空文件夹不计入
///
pub fn get_mod_info_from_dir(path: String) -> std::io::Result<Vec<ModFileInfo>> {
    // 取当前目录名为mod名称
    let mod_name = PathBuf::from(&path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut mod_files = vec![];

    // 循环读取目录下所有文件
    let mut file_list: Vec<DirEntry> = fs::read_dir(path)?
        .map(|it| it.expect("获取路径失败"))
        .collect();

    while let Some(file) = file_list.pop() {
        let metadata = file.metadata()?;
        // 为文件则创建实体对象
        if metadata.is_file() {
            let path = file.path().to_str().unwrap().to_string();
            mod_files.push(ModFileInfo::new(mod_name.clone(), path));
        }

        // 为目录则读取其子成员
        if metadata.is_dir() {
            for child in fs::read_dir(file.path())? {
                let child = child.expect(&format!("{:?}读取失败", file.file_name()));
                file_list.push(child);
            }
        }
    }
    Ok(mod_files)
}

/// 获取文件夹中所有子目录对于的mod信息，获得的mod信息为原始信息
/// 设计为启动时遍历mod文件夹以确定新增mod信息，暂未使用
///
pub fn get_mods_from_dir(path: impl Into<String>) -> Vec<ModInfo> {
    let path = path.into();
    let mut mods = vec![];
    let msg = format!("{:?} 读取失败", path);
    for entry in fs::read_dir(path).expect(&msg) {
        let entry = entry.expect(&msg);
        let child_path = entry.path().to_str().unwrap().to_string();
        let mod_files = get_mod_info_from_dir(child_path).expect(&msg);
        if mod_files.is_empty() {
            continue;
        }
        let name = mod_files[0].name.clone();
        let mut mod_info = ModInfo::new(name);
        mod_info.path = Some(mod_files.into_iter().map(|it| it.path).collect());
        mods.push(mod_info);
    }
    mods
}

/// 从单个文件目录中获取末端信息
///
pub fn get_mod_from_dir(dir_path: impl Into<String>) -> Option<ModInfo> {
    let dir_path = dir_path.into();

    let mod_files = get_mod_info_from_dir(dir_path.clone()).unwrap();
    if mod_files.is_empty() {
        return None;
    }
    let mut mod_info = ModInfo::new(mod_files.get(0).unwrap().name.clone());
    mod_info.path = Some(mod_files.into_iter().map(|it| it.path).collect());
    Some(mod_info)
}


/// 转移mod文件到MODFILE文件目录下，方便统一管理
///
pub fn add_mod_file(path: &String, mod_info: &mut ModInfo) {
    let name = PathBuf::from(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    // 获取当前mod地址 -> MODFILE管理地址映射
    let pattern = path.clone();
    let target = format!(".\\{}\\{}", MODFILE, name);
    let get_target_path = move |it: &String| it.replace(&pattern, &target);

    let mut target_paths = vec![];
    for source in mod_info.path.as_ref().unwrap() {
        let target_path = get_target_path(source);
        make_dir(&target_path);
        fs::File::create(&target_path).unwrap();
        fs::copy(source, &target_path).unwrap();
        target_paths.push(target_path);
    }
    mod_info.path = Some(target_paths);
}

/// 创建目录
///
pub fn make_dir(path: &String) {
    let target_parent = Path::new(path).parent().unwrap().to_str().unwrap();
    if !Path::new(target_parent).exists() {
        fs::create_dir_all(target_parent).unwrap();
    }
}

pub fn remove_all_game_file() {
    fs::remove_dir_all(format!("./{}", GAMEFILE)).unwrap();
}
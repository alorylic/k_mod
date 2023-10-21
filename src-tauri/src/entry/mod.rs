use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

mod data_format;

// mod文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFileInfo {
    pub name: String,
    pub path: String,
}

impl ModFileInfo {
    pub fn new(name: String, path: String) -> ModFileInfo {
        ModFileInfo { name, path }
    }
}

// mod信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub path: Option<Vec<String>>,
    pub apply: bool,
    
    #[serde(with = "data_format")]
    pub insert_time: DateTime<Local>,
}

impl ModInfo {
    pub fn new(name: String) -> ModInfo {
        ModInfo {
            name,
            path: None,
            apply: false,
            insert_time: Local::now(),
        }
    }
}

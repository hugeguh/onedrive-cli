use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use colored::*;

static CONFIG_FILE_PATH: &str = "config.yml";

#[derive(Serialize, Deserialize, Debug)]
pub struct OnedriveConfig {
    pub appid: String,
    pub secret: String,
    pub token_endpoint: String,
    pub ms_graph_scope: String,
    pub drive: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub onedrive: OnedriveConfig,
    pub files: Vec<String>
}

pub fn get_config() -> Config {
    if !Path::new(CONFIG_FILE_PATH).exists() {
        println!("{}", "❌ 配置文件config.yml不存在，请先创建配置文件".red());
        std::process::exit(0);
    } else {
        let config_file = File::open(CONFIG_FILE_PATH).unwrap();
        match serde_yaml::from_reader(config_file) {
            Ok(config @ Config { .. }) => {
                config
            }
            Err(e) => {
                eprintln!("{}", e);
                println!("{}", "❌ 配置文件config.yml格式错误，请检查配置文件".red());
                std::process::exit(0);
            }
        }
    }
}

pub enum PathType {
    FILE,
    DIR,
    NEITHER,
    ERROR
}

pub fn is_file_or_dir(p: &str) -> PathType {
    match fs::metadata(p) {
        Ok(metadata) => {
            if metadata.is_dir() {
                PathType::DIR
            } else if metadata.is_file() {
                PathType::FILE
            } else {
                PathType::NEITHER
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            PathType::ERROR
        }
    }
}

pub fn read_dir_files(dir_path: &str) -> Vec<PathBuf> {
    let mut list = vec![];

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    list.push(path)
                }
            }
        }
    } else {
        eprintln!("Error reading directory.");
    }
    list
}
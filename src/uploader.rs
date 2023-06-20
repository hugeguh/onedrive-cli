use std::path::Path;
use serde_json::json;
use colored::*;
use crate::config::Config;
use crate::auth::get_token;
use indicatif::ProgressBar;

static SLICE_SIZE: u32 = 5 * 1024 * 1024;
static MSAPI: &str = "https://graph.microsoft.com/v1.0/";

async fn create_request_instance(config: &Config) -> reqwest::Client {
    let token = get_token(config).await;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    client
}

async fn upload_slice(client: &reqwest::Client, slice: &[u8], upload_url: &str, bytes_range: &str) -> reqwest::Response {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::CONTENT_LENGTH, reqwest::header::HeaderValue::from_str(&format!("{}", slice.len())).unwrap());
    headers.insert(reqwest::header::CONTENT_RANGE, reqwest::header::HeaderValue::from_str(bytes_range).unwrap());
    let v = client.put(upload_url).headers(headers).body(slice.to_vec()).send().await.unwrap();
    v
}


async fn upload(file_path: &str, file_name: &str, config: &Config, client: &mut reqwest::Client) -> () {
    let file_size = Path::new(file_path).metadata().unwrap().len();
    let slice_count = file_size as f64 / SLICE_SIZE as f64;
    let slice_count = slice_count.ceil() as u64;
    
    let url = format!("{}{}{}{}:/createUploadSession", MSAPI, config.onedrive.drive, config.onedrive.folder, file_name);
    
    let body_data = json!(
        {
            "@microsoft.graph.conflictBehavior": "rename"
        }
    );
    dbg!(&url);
    println!("{} {}", "🚀 开始上传".green(), file_name);
    let v = client.post(url).body(body_data.to_string()).send().await.unwrap();
    match v.status() {
        reqwest::StatusCode::OK => {
            println!("{} {}", "✅ 创建上传会话成功".green(), file_name);
            let body = v.text().await.unwrap();
            let json: serde_json::Value = serde_json::from_str(&body).unwrap();
            let upload_url = json["uploadUrl"].as_str().unwrap();
            let file_buffer = std::fs::read(file_path).unwrap();

            let mut index = 0;
            let p_bar = ProgressBar::new(slice_count);
            while index < slice_count {
                let start = index * SLICE_SIZE as u64;
                let mut end = (index + 1) * SLICE_SIZE as u64;
                if end > file_size {
                    end = file_size;
                }
                let buffer_slice = &file_buffer[start as usize..end as usize];
                let bytes_range = format!("bytes {}-{}/{}", start, end - 1, file_size);
                
                let u_ret = upload_slice(client, buffer_slice, upload_url, &bytes_range).await;
                p_bar.inc(1);
                match u_ret.status() {
                    reqwest::StatusCode::CREATED => {
                    }
                    reqwest::StatusCode::ACCEPTED => {
                    }
                    reqwest::StatusCode::OK => {
                        println!("{}", "✅ 上传成功".green());
                    }
                    reqwest::StatusCode::UNAUTHORIZED => {
                        eprintln!("{}", "❌ 上传分片失败".red());
                        *client = create_request_instance(config).await;
                        continue
                    }
                    _ => {
                        eprintln!("{}", u_ret.status());
                        eprintln!("{}", u_ret.text().await.unwrap());
                        eprintln!("{}", "❌ 上传分片失败".red());
                    }
                }
                index += 1;
                
            }
            p_bar.finish_and_clear();
            ()
        }
        _ => {
            eprintln!("{}", v.text().await.unwrap());
            eprintln!("{} {}", "❌ 创建上传会话失败".red(), file_name);
            ()
        }
    }
    
}


async fn read_file(file_path: &str, config: &Config, client: &mut reqwest::Client) -> () {
    let file_name = Path::new(file_path).file_name();
    if let Some(file_name) = file_name {
        match file_name.to_str() {
            Some(name) => {
                upload(file_path, name, config, client).await;
                ()
            }
            None => {
                eprint!("❌ 读取文件失败：{}", file_path);
                ()
            }
        }
    } else {
        eprint!("❌ 读取文件失败：{}", file_path);
        ()
    }
}


pub async fn upload_files(config: &Config) {
    let mut client = create_request_instance(config).await;
    for file in &config.files {
        read_file(file, config, &mut client).await;
    }
}
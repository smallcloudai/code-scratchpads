use tracing::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock as ARwLock;
use std::sync::RwLock as StdRwLock;
use std::path::PathBuf;
use std::collections::HashMap;
// use reqwest_eventsource::Event;
use serde_json::json;
use crate::caps::CodeAssistantCaps;
use serde::Deserialize;
use serde::Serialize;
use crate::global_context;
// use chrono::Utc;


const TELEMETRY_COMPRESSION_SECONDS: u64 = 10;


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TelemetryNetwork {
    pub url: String,           // communication with url
    pub scope: String,         // in relation to what
    pub success: bool,
    pub error_message: String, // empty if no error
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TelemetryCompletion {
    pub language: String,
    pub multiline: bool,
    pub accepted: bool,
    pub user_pondered_600ms: bool,
    pub user_pondered_1200ms: bool,
    // -- key above --
    pub generated_chars: usize,
    pub remaining_percent: f64,
}


pub struct Storage {
    pub last_flushed_ts: i64,
    pub tele_net: Vec<TelemetryNetwork>,
    pub tele_comp: Vec<TelemetryCompletion>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            last_flushed_ts: 0,
            tele_net: Vec::new(),
            tele_comp: Vec::new(),
        }
    }
}

fn _key_telemetry_network(rec: &TelemetryNetwork) -> String {
    format!("{}/{}/{}/{}", rec.url, rec.scope, rec.success, rec.error_message)
}

fn _compress_telemetry_network(
    storage: Arc<StdRwLock<Storage>>,
) -> serde_json::Value {
    let mut key2cnt = HashMap::<String, i32>::new();
    let mut key2dict = HashMap::<String, serde_json::Value>::new();
    {
        let storage_locked = storage.write().unwrap();
        for rec in storage_locked.tele_net.iter() {
            let key = _key_telemetry_network(rec);
            if !key2dict.contains_key(&key) {
                key2dict.insert(key.clone(), serde_json::to_value(rec).unwrap());
                key2cnt.insert(key.clone(), 0);
            }
            key2cnt.insert(key.clone(), key2cnt[&key] + 1);
        }
    }
    let mut big_json_net = serde_json::json!({});
    for (key, cnt) in key2cnt.iter() {
        let mut json_dict = key2dict[key.as_str()].clone();
        json_dict["counter"] = json!(cnt);
        big_json_net.as_object_mut().unwrap().insert(key.clone(), json_dict);
    }
    big_json_net
}

fn _key_telemetry_completion(rec: &TelemetryCompletion) -> String {
    format!("{}/{}/{}/{}/{}", rec.language, rec.multiline, rec.accepted, rec.user_pondered_600ms, rec.user_pondered_1200ms)
}

pub async fn compress_basic_telemetry_to_file(
    cx: Arc<ARwLock<global_context::GlobalContext>>,
) {
    let cache_dir: PathBuf;
    let storage: Arc<StdRwLock<Storage>>;
    {
        let cx_locked = cx.read().await;
        storage = cx_locked.telemetry.clone();
        cache_dir = cx_locked.cache_dir.clone();
    }
    let mut big_json_net = _compress_telemetry_network(storage.clone());
    let dir = cache_dir.join("telemetry").join("compressed");
    tokio::fs::create_dir_all(dir.clone()).await.unwrap_or_else(|_| {});
    let now = chrono::Local::now();
    let fn_net = dir.join(format!("{}-net.json", now.format("%Y%m%d-%H%M%S")));
    big_json_net.as_object_mut().unwrap().insert("teletype".to_string(), json!("network"));
    big_json_net.as_object_mut().unwrap().insert("ts_end".to_string(), json!(now.timestamp()));
    {
        let mut storage_locked = storage.write().unwrap();
        storage_locked.tele_net.clear();
        storage_locked.tele_comp.clear();
        big_json_net.as_object_mut().unwrap().insert("ts_begin".to_string(), json!(storage_locked.last_flushed_ts));
        storage_locked.last_flushed_ts = now.timestamp();
    }
    // even if there's an error with i/o, storage is now clear, preventing infinite memory growth
    info!("basic telemetry saving \"{}\"", fn_net.to_str().unwrap());
    let mut f_net = tokio::fs::File::create(fn_net).await.unwrap();
    f_net.write_all(serde_json::to_string(&big_json_net).unwrap().as_bytes()).await.unwrap();
}

pub async fn cleanup_old_files(
    dir: PathBuf,
    how_much_to_keep: usize,
) {
    let files = {
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            let mut sorted = Vec::<PathBuf>::new();
            while let Some(entry) = entries.next_entry().await.unwrap() {
                if !entry.file_type().await.unwrap().is_file() {
                    continue;
                }
                let path = entry.path();
                if !path.to_str().unwrap().ends_with(".json") {
                    continue;
                }
                sorted.push(path);
            }
            sorted.sort_by(|a, b| b.cmp(&a));
            sorted
        } else {
            Vec::<PathBuf>::new()
        }
    };
    let mut leave_alone = how_much_to_keep;
    for path in files {
        leave_alone -= 1;
        if leave_alone > 0 {
            info!("leave_alone telemetry file: {}", path.to_str().unwrap());
            continue;
        }
        info!("removing old telemetry file: {}", path.to_str().unwrap());
        tokio::fs::remove_file(path).await.unwrap_or_else(|e| {
            error!("error removing old telemetry file: {}", e);
            // better to continue deleting, not much we can do
        });
    }
}

pub async fn send_telemetry_files_to_mothership(
    cx: Arc<ARwLock<global_context::GlobalContext>>,
    api_key: String,
    enduser_client_version: String,
) {
    unimplemented!();
}

pub async fn telemetry_background_task(
    global_context: Arc<ARwLock<global_context::GlobalContext>>,
) -> () {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(TELEMETRY_COMPRESSION_SECONDS)).await;
        info!("basic telemetry compression starts");
        let caps: Option<Arc<StdRwLock<CodeAssistantCaps>>>;
        let api_key: String;
        let enduser_client_version: String;
        let mothership_enabled: bool;
        let mut telemetry_basic_dest: String = String::new();
        let cache_dir: PathBuf;
        {
            let cx = global_context.write().await;
            caps = cx.caps.clone();
            cache_dir = cx.cache_dir.clone();
            api_key = cx.cmdline.api_key.clone();
            enduser_client_version = cx.cmdline.enduser_client_version.clone();
            mothership_enabled = cx.cmdline.basic_telemetry;
        }
        if caps.is_some() {
            telemetry_basic_dest = caps.unwrap().read().unwrap().telemetry_basic_dest.clone();
        }
        if !telemetry_basic_dest.is_empty() {
        }
        compress_basic_telemetry_to_file(global_context.clone()).await;
        if mothership_enabled {
            send_telemetry_files_to_mothership(global_context.clone(), api_key, enduser_client_version).await;
        }
        let dir_compressed = cache_dir.join("telemetry").join("compressed");
        let dir_sent = cache_dir.join("telemetry").join("sent");
        cleanup_old_files(dir_compressed, 10).await;
        cleanup_old_files(dir_sent, 10).await;
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use aws_sdk_s3::Client as S3Client;
use lru::LruCache;
use once_cell::sync::Lazy;

use crate::config::constants::BB_CHIP_VALUE;
use crate::types::s3::{S3Equity, S3Node, S3Settings, SolverTree};
use crate::DataSource;

static TREE_CACHE: Lazy<Mutex<LruCache<String, SolverTree>>> =
    Lazy::new(|| Mutex::new(LruCache::new(std::num::NonZeroUsize::new(16).unwrap())));

pub fn stacks_to_folder(stacks_bb: &[f64]) -> String {
    stacks_bb
        .iter()
        .map(|s| {
            let chips = (s * BB_CHIP_VALUE as f64).round() as u64;
            (chips / BB_CHIP_VALUE).to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Load tree from whichever source is configured
pub async fn load_tree(source: &DataSource, folder: &str) -> Result<SolverTree, String> {
    if let Some(cached) = cache_get(folder) {
        return Ok(cached);
    }

    let tree = match source {
        DataSource::Local { path } => load_tree_local(path, folder)?,
        DataSource::S3 { client, bucket, prefix } => {
            load_tree_s3(client, bucket, prefix, folder).await?
        }
    };

    cache_put(folder, &tree);
    Ok(tree)
}

// --- Local filesystem ---

fn load_tree_local(base_path: &Path, folder: &str) -> Result<SolverTree, String> {
    let folder_path = base_path.join(folder);
    if !folder_path.exists() {
        return Err(format!("Solution folder not found: {folder}"));
    }

    let settings: S3Settings = load_json_file(&folder_path.join("settings.json"))?;
    let equity: S3Equity = load_json_file(&folder_path.join("equity.json"))?;
    let nodes = load_all_nodes_local(&folder_path.join("nodes"))?;

    Ok(SolverTree { settings, equity, nodes })
}

fn load_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))
}

fn load_all_nodes_local(nodes_dir: &Path) -> Result<HashMap<u32, S3Node>, String> {
    let mut nodes = HashMap::new();

    let entries: Vec<PathBuf> = std::fs::read_dir(nodes_dir)
        .map_err(|e| format!("Failed to read nodes dir: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();

    for path in entries {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;
        let node_id: u32 = stem.parse().map_err(|_| format!("Invalid node ID: {stem}"))?;
        let node: S3Node = load_json_file(&path)?;
        nodes.insert(node_id, node);
    }

    Ok(nodes)
}

// --- AWS S3 ---

async fn load_tree_s3(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
    folder: &str,
) -> Result<SolverTree, String> {
    let base_key = if prefix.is_empty() {
        folder.to_string()
    } else {
        format!("{prefix}/{folder}")
    };

    log::info!("Loading tree from S3: s3://{bucket}/{base_key}");

    let settings: S3Settings =
        load_json_s3(client, bucket, &format!("{base_key}/settings.json")).await?;
    let equity: S3Equity =
        load_json_s3(client, bucket, &format!("{base_key}/equity.json")).await?;
    let nodes = load_all_nodes_s3(client, bucket, &format!("{base_key}/nodes")).await?;

    log::info!("Loaded {} nodes from S3", nodes.len());

    Ok(SolverTree { settings, equity, nodes })
}

async fn load_json_s3<T: serde::de::DeserializeOwned>(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<T, String> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| format!("S3 get {key}: {e}"))?;

    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| format!("S3 read body {key}: {e}"))?
        .into_bytes();

    serde_json::from_slice(&bytes).map_err(|e| format!("S3 parse {key}: {e}"))
}

async fn load_all_nodes_s3(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
) -> Result<HashMap<u32, S3Node>, String> {
    let mut nodes = HashMap::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(format!("{prefix}/"));

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("S3 list {prefix}: {e}"))?;

        for obj in resp.contents() {
            let key: &str = match obj.key() {
                Some(k) => k,
                None => continue,
            };
            if !key.ends_with(".json") {
                continue;
            }

            let filename = key.rsplit('/').next().unwrap_or_default();
            let stem = filename.strip_suffix(".json").unwrap_or_default();
            let node_id: u32 = match stem.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let node: S3Node = load_json_s3(client, bucket, key).await?;
            nodes.insert(node_id, node);
        }

        if resp.is_truncated() == Some(true) {
            continuation_token = resp.next_continuation_token().map(|s| s.to_string());
        } else {
            break;
        }
    }

    Ok(nodes)
}

// --- Cache helpers ---

fn cache_get(folder: &str) -> Option<SolverTree> {
    let mut cache = TREE_CACHE.lock().unwrap();
    cache.get(folder).cloned()
}

fn cache_put(folder: &str, tree: &SolverTree) {
    let mut cache = TREE_CACHE.lock().unwrap();
    cache.put(folder.to_string(), tree.clone());
}

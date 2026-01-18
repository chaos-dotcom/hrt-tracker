use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub const DATA_FILE_PATH: &str = "data/hrt-data.json";
pub const SETTINGS_FILE_PATH: &str = "data/hrt-settings.yaml";
pub const PHOTOS_DIR: &str = "data/dosage-photos";

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub async fn read_json<T: DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<Option<T>, StorageError> {
    let path = path.as_ref();
    let text = match fs::read_to_string(path).await {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    if text.trim().is_empty() {
        return Ok(None);
    }

    let parsed = serde_json::from_str(&text)?;
    Ok(Some(parsed))
}

pub async fn write_json_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), StorageError> {
    let path = path.as_ref();
    ensure_parent_dir(path).await?;

    let tmp_path = temp_path(path);
    let text = serde_json::to_string_pretty(value)?;

    let mut file = fs::File::create(&tmp_path).await?;
    file.write_all(text.as_bytes()).await?;
    file.sync_all().await?;
    fs::rename(&tmp_path, path).await?;

    Ok(())
}

pub async fn read_yaml<T: DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<Option<T>, StorageError> {
    let path = path.as_ref();
    let text = match fs::read_to_string(path).await {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    if text.trim().is_empty() {
        return Ok(None);
    }

    let parsed = serde_yaml::from_str(&text)?;
    Ok(Some(parsed))
}

pub async fn write_yaml<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), StorageError> {
    let path = path.as_ref();
    ensure_parent_dir(path).await?;

    let text = serde_yaml::to_string(value)?;
    fs::write(path, text).await?;

    Ok(())
}

pub async fn save_photo(
    entry_id: &str,
    filename: &str,
    bytes: &[u8],
) -> Result<PathBuf, StorageError> {
    let dir = Path::new(PHOTOS_DIR).join(entry_id);
    fs::create_dir_all(&dir).await?;
    let path = dir.join(filename);
    fs::write(&path, bytes).await?;
    Ok(path)
}

pub async fn read_photo(entry_id: &str, filename: &str) -> Result<Option<Vec<u8>>, StorageError> {
    let path = Path::new(PHOTOS_DIR).join(entry_id).join(filename);
    match fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_photo(entry_id: &str, filename: &str) -> Result<bool, StorageError> {
    let path = Path::new(PHOTOS_DIR).join(entry_id).join(filename);
    match fs::remove_file(path).await {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

pub fn content_type_from_ext(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "heic" => "image/heic",
        _ => "application/octet-stream",
    }
}

async fn ensure_parent_dir(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let mut buf = path.as_os_str().to_os_string();
    buf.push(".tmp");
    PathBuf::from(buf)
}

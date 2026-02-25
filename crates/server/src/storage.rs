use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sqlx::any::AnyPoolOptions;
use sqlx::{AnyPool, QueryBuilder, Row};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub const DATA_FILE_PATH: &str = "data/hrt-data.json";
pub const SETTINGS_FILE_PATH: &str = "data/hrt-settings.yaml";
pub const DEFAULT_DATABASE_URL: &str = "sqlite://./data/hrt-data.db?mode=rwc";
pub const PHOTOS_DIR: &str = "data/dosage-photos";
pub const BLOODTEST_PDFS_DIR: &str = "data/bloodtest-pdfs";

const DATA_KEY: &str = "data";
const SETTINGS_KEY: &str = "settings";

#[derive(Clone)]
struct DbStore {
    pool: AnyPool,
}

static DB_STORE: OnceLock<DbStore> = OnceLock::new();

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("storage init error: {0}")]
    Init(String),
}

pub async fn initialize_storage() -> Result<(), StorageError> {
    if DB_STORE.get().is_some() {
        return Ok(());
    }

    let database_url = std::env::var("HRT_DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string())
        .trim()
        .to_string();
    if database_url.is_empty() {
        return Err(StorageError::Init(
            "HRT_DATABASE_URL must not be empty".to_string(),
        ));
    }

    sqlx::any::install_default_drivers();
    ensure_sqlite_parent_dir(&database_url).await?;

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    init_schema(&pool).await?;

    let store = DbStore { pool };
    import_legacy_files_if_needed(&store).await?;
    sync_backup_files(&store).await?;

    let _ = DB_STORE.set(store);
    Ok(())
}

pub async fn read_data_value() -> Result<Option<Value>, StorageError> {
    if let Some(store) = DB_STORE.get() {
        return read_db_json(store, DATA_KEY).await;
    }
    read_json(DATA_FILE_PATH).await
}

pub async fn write_data_value(value: &Value) -> Result<(), StorageError> {
    if let Some(store) = DB_STORE.get() {
        write_db_json(store, DATA_KEY, value).await?;
        write_json_atomic(DATA_FILE_PATH, value).await?;
        return Ok(());
    }
    write_json_atomic(DATA_FILE_PATH, value).await
}

pub async fn read_settings_value() -> Result<Option<Value>, StorageError> {
    if let Some(store) = DB_STORE.get() {
        return read_db_json(store, SETTINGS_KEY).await;
    }
    read_yaml(SETTINGS_FILE_PATH).await
}

pub async fn write_settings_value(value: &Value) -> Result<(), StorageError> {
    if let Some(store) = DB_STORE.get() {
        write_db_json(store, SETTINGS_KEY, value).await?;
        write_yaml(SETTINGS_FILE_PATH, value).await?;
        return Ok(());
    }
    write_yaml(SETTINGS_FILE_PATH, value).await
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

pub async fn save_bloodtest_pdf(filename: &str, bytes: &[u8]) -> Result<PathBuf, StorageError> {
    let dir = Path::new(BLOODTEST_PDFS_DIR);
    fs::create_dir_all(dir).await?;
    let path = dir.join(filename);
    fs::write(&path, bytes).await?;
    Ok(path)
}

pub async fn read_bloodtest_pdf(filename: &str) -> Result<Option<Vec<u8>>, StorageError> {
    let path = Path::new(BLOODTEST_PDFS_DIR).join(filename);
    match fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_bloodtest_pdf(filename: &str) -> Result<bool, StorageError> {
    let path = Path::new(BLOODTEST_PDFS_DIR).join(filename);
    match fs::remove_file(path).await {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

pub fn content_type_from_ext(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "pdf" => "application/pdf",
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

async fn ensure_sqlite_parent_dir(database_url: &str) -> Result<(), StorageError> {
    let is_sqlite = database_url.starts_with("sqlite:");
    if !is_sqlite || database_url.contains(":memory:") {
        return Ok(());
    }

    let path = if let Some(rest) = database_url.strip_prefix("sqlite://") {
        rest
    } else if let Some(rest) = database_url.strip_prefix("sqlite:") {
        rest
    } else {
        ""
    };

    let path = path.split('?').next().unwrap_or("");
    let path = path.trim();
    if path.is_empty() {
        return Ok(());
    }

    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

async fn init_schema(pool: &AnyPool) -> Result<(), StorageError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS hrt_store (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at BIGINT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn import_legacy_files_if_needed(store: &DbStore) -> Result<(), StorageError> {
    let data_present = read_db_json(store, DATA_KEY).await?.is_some();
    let settings_present = read_db_json(store, SETTINGS_KEY).await?.is_some();

    if !data_present {
        match read_json::<Value>(DATA_FILE_PATH).await {
            Ok(Some(legacy_data)) => write_db_json(store, DATA_KEY, &legacy_data).await?,
            Ok(None) | Err(StorageError::Json(_)) => {}
            Err(err) => return Err(err),
        }
    }

    if !settings_present {
        match read_yaml::<Value>(SETTINGS_FILE_PATH).await {
            Ok(Some(legacy_settings)) => {
                write_db_json(store, SETTINGS_KEY, &legacy_settings).await?
            }
            Ok(None) | Err(StorageError::Yaml(_)) => {}
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

async fn sync_backup_files(store: &DbStore) -> Result<(), StorageError> {
    if let Some(data_value) = read_db_json(store, DATA_KEY).await? {
        write_json_atomic(DATA_FILE_PATH, &data_value).await?;
    }

    if let Some(settings_value) = read_db_json(store, SETTINGS_KEY).await? {
        write_yaml(SETTINGS_FILE_PATH, &settings_value).await?;
    }

    Ok(())
}

async fn read_db_json(store: &DbStore, key: &str) -> Result<Option<Value>, StorageError> {
    let mut query = QueryBuilder::new("SELECT value FROM hrt_store WHERE key = ");
    query.push_bind(key);
    let row = query.build().fetch_optional(&store.pool).await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let raw: String = row.try_get("value")?;
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let parsed: Value = serde_json::from_str(&raw)?;
    Ok(Some(parsed))
}

async fn write_db_json(store: &DbStore, key: &str, value: &Value) -> Result<(), StorageError> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let payload = serde_json::to_string(value)?;

    let mut query = QueryBuilder::new(
        "INSERT INTO hrt_store (key, value, updated_at) \
         VALUES (",
    );
    query.push_bind(key);
    query.push(", ");
    query.push_bind(payload);
    query.push(", ");
    query.push_bind(now_ms);
    query.push(
        ") ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    );

    query.build().execute(&store.pool).await?;

    Ok(())
}

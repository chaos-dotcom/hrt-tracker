use axum::body::Bytes;
use axum::extract::{Multipart, Path};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use lopdf::Document;
use serde_json::{json, Value};

use hrt_shared::convert::convert_hormone;
use hrt_shared::types::Hormone;

use crate::storage::{
    content_type_from_ext, delete_bloodtest_pdf as delete_bloodtest_pdf_file, delete_photo,
    read_bloodtest_pdf as read_bloodtest_pdf_file, read_data_value, read_photo,
    read_settings_value, save_bloodtest_pdf, save_photo, write_data_value, write_settings_value,
};

pub async fn get_data() -> Response {
    match read_data_value().await {
        Ok(Some(mut value)) => {
            if !value.is_object() {
                value = json!({});
            }
            if let Some(obj) = value.as_object_mut() {
                obj.remove("settings");
            }
            Json(value).into_response()
        }
        Ok(None) => Json(json!({})).into_response(),
        Err(err) => match err {
            crate::storage::StorageError::Json(_) => Json(json!({})).into_response(),
            _ => json_error("Failed to read data", StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub async fn post_data(body: Bytes) -> Response {
    let mut value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return json_error("Failed to write data", StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let Some(obj) = value.as_object_mut() {
        obj.remove("settings");
    }

    if let Err(_) = write_data_value(&value).await {
        return json_error("Failed to write data", StatusCode::INTERNAL_SERVER_ERROR);
    }

    Json(json!({ "success": true })).into_response()
}

pub async fn get_settings() -> Response {
    match read_settings_value().await {
        Ok(Some(mut value)) => {
            if !value.is_object() {
                value = json!({});
            }
            Json(value).into_response()
        }
        Ok(None) => Json(json!({})).into_response(),
        Err(_) => json_error("Failed to read settings", StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn post_settings(body: Bytes) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return json_error(
                "Failed to write settings",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };

    let payload = if value.is_object() { value } else { json!({}) };

    if let Err(_) = write_settings_value(&payload).await {
        return json_error(
            "Failed to write settings",
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    Json(json!({ "success": true })).into_response()
}

pub async fn convert(body: Bytes) -> Response {
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return json_error("Conversion failed", StatusCode::INTERNAL_SERVER_ERROR),
    };

    let value = payload.get("value").and_then(|v| v.as_f64());
    let hormone_value = payload.get("hormone").and_then(|v| v.as_str());
    let from_unit = payload.get("fromUnit").and_then(|v| v.as_str());
    let to_unit = payload.get("toUnit").and_then(|v| v.as_str());

    let value = match value {
        Some(value) if value.is_finite() => value,
        _ => return json_error("Invalid value", StatusCode::BAD_REQUEST),
    };

    let hormone_str = match hormone_value {
        Some(value) => value,
        None => return json_error("Conversion failed", StatusCode::BAD_REQUEST),
    };
    let from_unit = match from_unit {
        Some(value) => value,
        None => return json_error("Conversion failed", StatusCode::BAD_REQUEST),
    };
    let to_unit = match to_unit {
        Some(value) => value,
        None => return json_error("Conversion failed", StatusCode::BAD_REQUEST),
    };

    let hormone: Hormone = match serde_json::from_value(Value::String(hormone_str.to_string())) {
        Ok(value) => value,
        Err(err) => return json_error(&err.to_string(), StatusCode::BAD_REQUEST),
    };

    match convert_hormone(value, hormone, from_unit, to_unit) {
        Ok(converted) => {
            let rounded = (converted * 1000.0).round() / 1000.0;
            Json(json!({ "value": rounded, "unit": to_unit })).into_response()
        }
        Err(err) => json_error(&err, StatusCode::BAD_REQUEST),
    }
}

pub async fn upload_dosage_photo(
    Path(entry_id): Path<String>,
    mut multipart: Multipart,
) -> Response {
    if entry_id.trim().is_empty() {
        return json_error("missing entryId", StatusCode::BAD_REQUEST);
    }

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        if name != "file" && name != "photos" && name != "photo" {
            continue;
        }
        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if bytes.is_empty() {
            continue;
        }
        let ext = ext_from_name_or_type(filename.as_deref(), content_type.as_deref());
        let stored_name = format!("{}_{}.{}", Utc::now().timestamp_millis(), files.len(), ext);
        files.push((stored_name, bytes.to_vec()));
    }

    if files.is_empty() {
        return json_error("no files", StatusCode::BAD_REQUEST);
    }

    let mut filenames = Vec::new();
    for (filename, bytes) in files {
        if save_photo(&entry_id, &filename, &bytes).await.is_ok() {
            filenames.push(filename);
        }
    }

    Json(json!({ "filenames": filenames })).into_response()
}

pub async fn get_dosage_photo(Path((entry_id, filename)): Path<(String, String)>) -> Response {
    if entry_id.trim().is_empty() || filename.trim().is_empty() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".into())
            .unwrap();
    }

    let data = match read_photo(&entry_id, &filename).await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not found".into())
                .unwrap()
        }
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error".into())
                .unwrap()
        }
    };

    let ext = filename.split('.').last().unwrap_or("");
    let mut headers = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(content_type_from_ext(ext)) {
        headers.insert("Content-Type", value);
    }
    (StatusCode::OK, headers, data).into_response()
}

pub async fn delete_dosage_photo(Path((entry_id, filename)): Path<(String, String)>) -> Response {
    if entry_id.trim().is_empty() || filename.trim().is_empty() {
        return json_error("missing params", StatusCode::BAD_REQUEST);
    }

    match delete_photo(&entry_id, &filename).await {
        Ok(_) => Json(json!({ "success": true })).into_response(),
        Err(_) => json_error("delete failed", StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn upload_bloodtest_pdf(mut multipart: Multipart) -> Response {
    let settings = match read_settings_value().await {
        Ok(Some(value)) if value.is_object() => value,
        _ => json!({}),
    };
    let pdf_password = pdf_password_from_settings(&settings);

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        if name != "file" && name != "files" && name != "pdf" && name != "pdfs" {
            continue;
        }
        let filename = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if bytes.is_empty() {
            continue;
        }
        let ext = ext_from_name_or_type(filename.as_deref(), content_type.as_deref());
        if ext != "pdf" {
            continue;
        }
        let stored_name = format!("{}_{}.{}", Utc::now().timestamp_millis(), files.len(), ext);
        files.push((stored_name, bytes.to_vec()));
    }

    if files.is_empty() {
        return json_error("no files", StatusCode::BAD_REQUEST);
    }

    let mut payload = Vec::new();
    for (filename, bytes) in files {
        if save_bloodtest_pdf(&filename, &bytes).await.is_err() {
            continue;
        }
        let extraction = extract_pdf_text(&bytes, pdf_password.as_deref());
        let (text, extract_error) = match extraction {
            Ok(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    (None, Some("no extractable text found".to_string()))
                } else {
                    let capped: String = text.chars().take(120_000).collect();
                    (Some(capped), None)
                }
            }
            Err(err) => (None, Some(err)),
        };
        payload.push(json!({
            "filename": filename,
            "text": text,
            "extractError": extract_error,
        }));
    }

    Json(json!({ "files": payload })).into_response()
}

pub async fn get_bloodtest_pdf(Path(filename): Path<String>) -> Response {
    if !is_safe_storage_name(&filename) {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".into())
            .unwrap();
    }

    let data = match read_bloodtest_pdf_file(&filename).await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not found".into())
                .unwrap()
        }
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error".into())
                .unwrap()
        }
    };

    let ext = filename.split('.').last().unwrap_or("");
    let mut headers = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(content_type_from_ext(ext)) {
        headers.insert("Content-Type", value);
    }
    (StatusCode::OK, headers, data).into_response()
}

pub async fn delete_bloodtest_pdf(Path(filename): Path<String>) -> Response {
    if !is_safe_storage_name(&filename) {
        return json_error("invalid filename", StatusCode::BAD_REQUEST);
    }

    match delete_bloodtest_pdf_file(&filename).await {
        Ok(_) => Json(json!({ "success": true })).into_response(),
        Err(_) => json_error("delete failed", StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn ext_from_name_or_type(name: Option<&str>, content_type: Option<&str>) -> String {
    if let Some(name) = name {
        if let Some(ext) = name.split('.').last() {
            if !ext.is_empty() {
                return ext.to_lowercase();
            }
        }
    }

    match content_type.unwrap_or("") {
        "application/pdf" => "pdf".to_string(),
        "image/jpeg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/webp" => "webp".to_string(),
        "image/heic" => "heic".to_string(),
        _ => "bin".to_string(),
    }
}

fn pdf_password_from_settings(settings: &Value) -> Option<String> {
    settings
        .get("pdfPassword")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn extract_pdf_text(bytes: &[u8], password: Option<&str>) -> Result<String, String> {
    let mut document = match password.filter(|value| !value.trim().is_empty()) {
        Some(password) => Document::load_mem_with_password(bytes, password)
            .or_else(|_| Document::load_mem(bytes))
            .map_err(|err| err.to_string())?,
        None => Document::load_mem(bytes).map_err(|err| err.to_string())?,
    };

    if document.is_encrypted() && document.encryption_state.is_none() {
        if let Some(password) = password.filter(|value| !value.trim().is_empty()) {
            document.decrypt(password).map_err(|err| err.to_string())?;
        }
    }

    let pages = document.get_pages();
    if pages.is_empty() {
        return Ok(String::new());
    }
    let page_numbers: Vec<u32> = pages.keys().cloned().collect();
    document
        .extract_text(&page_numbers)
        .map_err(|err| err.to_string())
}

fn is_safe_storage_name(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains("..") {
        return false;
    }
    trimmed
        .bytes()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == b'.' || ch == b'_' || ch == b'-')
}

fn json_error(message: &str, status: StatusCode) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

use axum::http::StatusCode;
use axum::Json;

/// Server-side settings (stored in config file)
pub async fn get_settings() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let config_path = crate::file_store::config_dir().join("settings.json");
    let settings: serde_json::Value = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path).map_err(err)?;
        serde_json::from_str(&contents).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    Ok(Json(settings))
}

pub async fn save_settings(
    Json(settings): Json<serde_json::Value>,
) -> Result<Json<()>, (StatusCode, String)> {
    let config_dir = crate::file_store::config_dir();
    std::fs::create_dir_all(&config_dir).map_err(err)?;
    let config_path = config_dir.join("settings.json");
    let json = serde_json::to_string_pretty(&settings).map_err(err)?;
    std::fs::write(&config_path, json).map_err(err)?;
    Ok(Json(()))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

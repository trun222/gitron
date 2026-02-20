use crate::ai::error::AIResult;
use crate::ai::types::{AIModel, AIProvider, AISettings, GenerateResult};
use crate::ai::{credential, generate, providers};

#[tauri::command]
pub async fn ai_get_providers() -> AIResult<Vec<AIProvider>> {
    Ok(providers::get_providers())
}

#[tauri::command]
pub async fn ai_save_key(provider: String, key: String) -> AIResult<()> {
    credential::store_key(&provider, &key)
}

#[tauri::command]
pub async fn ai_delete_key(provider: String) -> AIResult<()> {
    credential::delete_key(&provider)
}

#[tauri::command]
pub async fn ai_fetch_models(
    provider: String,
    base_url: Option<String>,
) -> AIResult<Vec<AIModel>> {
    providers::fetch_models(&provider, base_url.as_deref()).await
}

#[tauri::command]
pub async fn ai_generate_commit_message(
    path: String,
    provider: String,
    model: String,
    base_url: Option<String>,
) -> AIResult<GenerateResult> {
    generate::generate_commit_message(&path, &provider, &model, base_url.as_deref()).await
}

#[tauri::command]
pub async fn ai_get_settings(
    app: tauri::AppHandle,
) -> AIResult<AISettings> {
    use tauri_plugin_store::StoreExt;
    let store = app
        .store("settings.json")
        .map_err(|e| crate::ai::error::AIError::ApiError(format!("Store error: {}", e)))?;
    let settings: AISettings = store
        .get("aiSettings")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    Ok(settings)
}

#[tauri::command]
pub async fn ai_save_settings(
    app: tauri::AppHandle,
    settings: AISettings,
) -> AIResult<()> {
    use tauri_plugin_store::StoreExt;
    let store = app
        .store("settings.json")
        .map_err(|e| crate::ai::error::AIError::ApiError(format!("Store error: {}", e)))?;
    let value = serde_json::to_value(&settings)
        .map_err(|e| crate::ai::error::AIError::ApiError(format!("Serialize error: {}", e)))?;
    store.set("aiSettings", value);
    Ok(())
}

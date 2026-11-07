mod native_model;
mod ocr;
mod research;
mod tool_registry;

#[tauri::command]
fn get_app_status() -> tredalix_ai::desktop::DesktopStatus {
    tredalix_ai::desktop::desktop_status()
}

#[tauri::command]
fn research_status() -> research::ResearchStatus {
    research::status()
}

#[tauri::command]
async fn web_research(queries: Vec<String>) -> Result<research::ResearchResult, String> {
    research::search(queries).await
}

#[tauri::command]
async fn recognize_screen_text(image: Vec<u8>) -> Result<ocr::OcrOutput, String> {
    tauri::async_runtime::spawn_blocking(move || ocr::recognize_png(&image))
        .await
        .map_err(|_| "The local OCR worker could not be completed.".to_string())?
}

#[tauri::command]
fn native_model_status(
    app: tauri::AppHandle,
    manager: tauri::State<'_, native_model::NativeModelManager>,
    model_dir: Option<String>,
) -> Result<native_model::NativeModelStatus, String> {
    native_model::status(&app, &manager, model_dir)
}

#[tauri::command]
fn verify_native_model(
    app: tauri::AppHandle,
    model_dir: Option<String>,
) -> Result<native_model::NativeModelStatus, String> {
    native_model::verify(&app, model_dir)
}

#[tauri::command]
async fn download_native_model(
    app: tauri::AppHandle,
    model_dir: Option<String>,
) -> Result<native_model::NativeModelStatus, String> {
    native_model::download(&app, model_dir).await
}

#[tauri::command]
fn remove_native_model(
    app: tauri::AppHandle,
    manager: tauri::State<'_, native_model::NativeModelManager>,
    model_dir: Option<String>,
) -> Result<native_model::NativeModelStatus, String> {
    native_model::remove(&app, &manager, model_dir)
}

#[tauri::command]
async fn load_native_model(
    app: tauri::AppHandle,
    manager: tauri::State<'_, native_model::NativeModelManager>,
    model_dir: Option<String>,
) -> Result<native_model::NativeModelStatus, String> {
    native_model::load(&app, &manager, model_dir).await
}

#[tauri::command]
fn unload_native_model(
    manager: tauri::State<'_, native_model::NativeModelManager>,
) -> Result<(), String> {
    native_model::unload(&manager)
}

#[tauri::command]
async fn native_model_chat(
    manager: tauri::State<'_, native_model::NativeModelManager>,
    messages: Vec<native_model::NativeChatMessage>,
    image: Option<Vec<u8>>,
    on_event: tauri::ipc::Channel<native_model::NativeChatEvent>,
) -> Result<native_model::NativeChatReply, String> {
    native_model::chat(&manager, messages, image, Some(on_event)).await
}

#[tauri::command]
fn list_ai_tools() -> Vec<tool_registry::ToolDefinition> {
    tool_registry::definitions()
}

#[tauri::command]
fn execute_ai_tool(
    id: String,
    arguments: serde_json::Value,
) -> Result<tool_registry::ToolExecution, String> {
    tool_registry::execute(&id, arguments)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(native_model::NativeModelManager::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            native_model_status,
            verify_native_model,
            download_native_model,
            remove_native_model,
            load_native_model,
            unload_native_model,
            native_model_chat,
            research_status,
            web_research,
            recognize_screen_text,
            list_ai_tools,
            execute_ai_tool
        ])
        .run(tauri::generate_context!())
        .expect("TREDALIX AI desktop runtime could not start");
}

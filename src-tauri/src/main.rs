#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use munhang_core::runtime;
use munhang_core::{export_hwpx as export, Project};
use std::io::Write;
use tauri_plugin_dialog::DialogExt;

fn save_bytes(
    app: tauri::AppHandle,
    bytes: Vec<u8>,
    extension: &str,
    filename: &str,
) -> Result<Option<String>, String> {
    let Some(file) = app
        .dialog()
        .file()
        .add_filter(extension, &[extension])
        .set_file_name(filename)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let folder = path.parent().ok_or("저장 폴더를 찾을 수 없습니다.")?;
    let mut temporary = tempfile::NamedTempFile::new_in(folder).map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(&path).map_err(|e| e.to_string())?;
    Ok(Some(path.to_string_lossy().into_owned()))
}
#[tauri::command]
async fn export_hwpx_to_file(
    app: tauri::AppHandle,
    project: Project,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = export(&project)?;
        save_bytes(app, bytes, "hwpx", "worksheet.hwpx")
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn save_recovery_text_to_file(
    app: tauri::AppHandle,
    text: String,
) -> Result<Option<String>, String> {
    if text.len() > 80 * 1024 * 1024 {
        return Err("복구 원문은 80 MB 이하여야 합니다.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        save_bytes(app, text.into_bytes(), "json", "worksheet-recovery.json")
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn save_project_to_file(
    app: tauri::AppHandle,
    project: Project,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Draft projects may contain unfinished equation scripts. Preserve them.
        let bytes = serde_json::to_vec_pretty(&project).map_err(|e| e.to_string())?;
        if bytes.len() > 80 * 1024 * 1024 {
            return Err("프로젝트는 80 MB 이하여야 합니다.".into());
        }
        save_bytes(app, bytes, "json", "worksheet.munhang.json")
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn export_hwpx(project: Project) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || export(&project))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn import_pdf(data: Vec<u8>) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || runtime::import_pdf(&data))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn recognize_image(data: Vec<u8>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || runtime::recognize_image(&data))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn recognize_formula(
    data: Vec<u8>,
) -> Result<munhang_core::formula::FormulaConversion, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let latex = runtime::recognize_formula(&data)?;
        Ok(munhang_core::formula::convert_latex(&latex))
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn convert_formula(latex: String) -> munhang_core::formula::FormulaConversion {
    munhang_core::formula::convert_latex(&latex)
}
#[tauri::command]
async fn runtime_status() -> Result<runtime::RuntimeStatus, String> {
    tauri::async_runtime::spawn_blocking(runtime::runtime_status)
        .await
        .map_err(|e| e.to_string())
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            export_hwpx_to_file,
            save_project_to_file,
            save_recovery_text_to_file,
            export_hwpx,
            import_pdf,
            recognize_image,
            recognize_formula,
            convert_formula,
            runtime_status
        ])
        .run(tauri::generate_context!())
        .expect("문항공방 데스크톱 실행 실패");
}

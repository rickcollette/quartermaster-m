mod atr;
mod basic;
mod basic_native;
mod document;
mod update;
mod version;

use tauri::Manager;

use atr::{
    atr_add_host_file, atr_close, atr_copy_file, atr_create, atr_delete_entry, atr_delete_file,
    atr_export_ascii, atr_extract_file, atr_import_host_files, atr_mkdir, atr_mount,
    atr_open_document, atr_rename_entry, atr_select_drive, atr_status, atr_write_document,
    local_open_folder, AtrState,
};
use basic::{
    basic_detokenize_atr, basic_detokenize_host, basic_save_listing_host,
    basic_save_listing_to_atr, basic_tokenize_host, basic_tokenize_to_atr,
};
use document::{load_document, save_document};
use update::{
    check_for_updates, download_and_install_update, download_macos_update, download_portable_update,
};

#[tauri::command]
fn app_version() -> &'static str {
    version::APP_VERSION
}

#[tauri::command]
fn app_ready(app: tauri::AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main application window was not created".to_string())?;
    main.show().map_err(|error| error.to_string())?;
    main.set_focus().map_err(|error| error.to_string())?;
    if let Some(splash) = app.get_webview_window("splashscreen") {
        splash.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AtrState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            app_version,
            app_ready,
            load_document,
            save_document,
            check_for_updates,
            download_portable_update,
            download_and_install_update,
            download_macos_update,
            atr_create,
            atr_mount,
            atr_select_drive,
            atr_status,
            atr_close,
            local_open_folder,
            atr_write_document,
            atr_add_host_file,
            atr_import_host_files,
            atr_copy_file,
            atr_extract_file,
            atr_export_ascii,
            atr_delete_file,
            atr_delete_entry,
            atr_rename_entry,
            atr_open_document,
            atr_mkdir,
            basic_detokenize_host,
            basic_tokenize_host,
            basic_detokenize_atr,
            basic_tokenize_to_atr,
            basic_save_listing_host,
            basic_save_listing_to_atr
        ])
        .run(tauri::generate_context!())
        .expect("error while running QuarterMaster/M");
}

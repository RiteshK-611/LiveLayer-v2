#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod types;
mod state;
mod utils;
mod platform;
mod tray;

use state::AppState;
use commands::*;
use tray::create_tray_menu;
use tauri::Manager;

fn main() {
    let app_state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(app_state)
        .setup(|app| {
            create_tray_menu(app)?;
            
            // Load persistent state and restore previous session
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(state) = commands::load_app_state(app_handle.clone()).await {
                    // Restore wallpaper if exists
                    if let (Some(wallpaper_path), Some(file_type)) = (&state.last_wallpaper_path, &state.last_wallpaper_file_type) {
                        let is_video = ["mp4", "webm", "avi", "mov", "mkv", "gif"].contains(&file_type.to_lowercase().as_str());
                        
                        if is_video {
                            let converted_path = format!("asset://localhost/{}", urlencoding::encode(wallpaper_path));
                            if let Some(app_state) = app_handle.try_state::<AppState>() {
                                let _ = commands::create_video_wallpaper(
                                    app_handle.clone(),
                                    wallpaper_path.clone(),
                                    converted_path,
                                    app_state,
                                ).await;
                            }
                        } else {
                            let _ = commands::set_static_wallpaper(wallpaper_path.clone()).await;
                        }
                    }
                    
                    // Restore date widget if enabled
                    if let Some(widget_settings) = &state.date_widget_settings {
                        if widget_settings.enabled {
                            if let Some(app_state) = app_handle.try_state::<AppState>() {
                                let _ = commands::create_date_widget(
                                    app_handle.clone(),
                                    app_state,
                                    widget_settings.clone(),
                                ).await;
                            }
                        }
                    }
                }
                
                // Show main window only if not started minimized
                let args: Vec<String> = std::env::args().collect();
                if !args.contains(&"--minimized".to_string()) {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_static_wallpaper,
            create_video_wallpaper,
            stop_video_wallpaper,
            get_wallpaper_files,
            get_files_info,
            show_main_window,
            hide_main_window,
            create_date_widget,
            hide_date_widget,
            show_date_widget,
            close_date_widget,
            update_date_widget,
            commands::app_state::save_app_state,
            commands::app_state::load_app_state,
            commands::app_state::set_autostart,
            commands::app_state::get_autostart_status,
            commands::app_state::update_wallpaper_state,
            commands::app_state::update_date_widget_state,
            commands::app_state::update_wallpaper_settings_state,
            commands::app_state::save_wallpaper_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
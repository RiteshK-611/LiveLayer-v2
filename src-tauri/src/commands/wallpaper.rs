use std::path::PathBuf;
use tauri::{AppHandle, State, Wry};
use crate::state::AppState;
use crate::utils::file_utils::{get_mime_type, is_gif_type};
use crate::commands::update_wallpaper_state;
use tauri::Manager;

#[cfg(target_os = "windows")]
use crate::platform::windows::set_wallpaper_behind_desktop_sync;

#[tauri::command]
pub async fn set_static_wallpaper(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::ffi::CString;
        use winapi::um::winuser::{SystemParametersInfoA, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE};
        
        let path_cstring = CString::new(file_path.clone())
            .map_err(|_| "Invalid file path".to_string())?;
        
        unsafe {
            let result = SystemParametersInfoA(
                SPI_SETDESKWALLPAPER,
                0,
                path_cstring.as_ptr() as *mut _,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
            
            if result == 0 {
                return Err("Failed to set wallpaper".to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!("tell application \"Finder\" to set desktop picture to POSIX file \"{}\"", file_path))
            .output()
            .map_err(|e| format!("Failed to execute osascript: {}", e))?;

        if !output.status.success() {
            return Err("Failed to set wallpaper on macOS".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        let desktop_commands = [
            ("gsettings", vec!["set", "org.gnome.desktop.background", "picture-uri", &format!("file://{}", file_path)]),
            ("feh", vec!["--bg-fill", &file_path]),
            ("nitrogen", vec!["--set-scaled", &file_path]),
            ("xfconf-query", vec!["-c", "xfce4-desktop", "-p", "/backdrop/screen0/monitor0/workspace0/last-image", "-s", &file_path]),
        ];

        let mut success = false;
        for (cmd, args) in desktop_commands.iter() {
            if Command::new(cmd).args(args).output().is_ok() {
                success = true;
                break;
            }
        }
        
        if !success {
            return Err("Failed to set wallpaper on Linux".to_string());
        }
    }

    Ok(format!("Wallpaper set successfully: {}", file_path))
}

#[tauri::command]
pub async fn create_video_wallpaper(
    app: AppHandle<Wry>,
    file_path: String,
    converted_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err("Video file does not exist".to_string());
    }

    // Create unique window label
    let window_label = format!("wallpaper-{}", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Handle video windows state
    {
        let mut video_windows = state.video_windows.lock().unwrap();
        if let Some(existing_label) = video_windows.get("current") {
            if let Some(window) = app.get_webview_window(existing_label) {
                let _ = window.close();
            }
        }
        video_windows.insert("current".to_string(), window_label.clone());
    }

    let file_extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let mime_type = if is_gif_type(&file_extension) {
        "image/gif".to_string()
    } else {
        get_mime_type(&file_path)
    };

    // Create wallpaper window URL with parameters
    let wallpaper_url = format!(
        "wallpaper.html?path={}&type={}",
        urlencoding::encode(&converted_path),
        urlencoding::encode(&mime_type)
    );

    // Create wallpaper window
    let video_window = tauri::WebviewWindowBuilder::new(
        &app,
        &window_label,
        tauri::WebviewUrl::App(wallpaper_url.into()),
    )
    .title("Video Wallpaper")
    .minimizable(false)
    .maximizable(false)
    .closable(false)
    .resizable(false)
    .decorations(false)
    .shadow(false)
    .visible(false)
    .skip_taskbar(true)
    .fullscreen(true)
    .build()
    .map_err(|e| format!("Failed to create wallpaper window: {}", e))?;

    // Set window to always be on bottom
    video_window.set_always_on_bottom(true)
        .map_err(|e| format!("Failed to set always on bottom: {}", e))?;

    // Show window after setup
    video_window.show()
        .map_err(|e| format!("Failed to show window: {}", e))?;

    // Wait for window to be ready
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Windows-specific: Use blocking task to avoid Send issues
    #[cfg(target_os = "windows")]
    {
        let video_window_clone = video_window.clone();
        
        // Spawn blocking task for Windows API calls - no Send trait required
        let result = tokio::task::spawn_blocking(move || {
            set_wallpaper_behind_desktop_sync(&video_window_clone)
        }).await;
        
        match result {
            Ok(Ok(_)) => {
                #[cfg(debug_assertions)]
                println!("Successfully set wallpaper behind desktop");
            }
            Ok(Err(_e)) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to set wallpaper behind desktop: {}", _e);
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to execute desktop integration task: {}", _e);
            }
        }
    }

    // Save wallpaper state
    let file_extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    let _ = update_wallpaper_state(
        app.clone(),
        file_path.clone(),
        file_extension,
    ).await;

    Ok(format!("Video wallpaper created successfully: {}", file_path))
}

#[tauri::command]
pub async fn stop_video_wallpaper(state: State<'_, AppState>, app: AppHandle<Wry>) -> Result<String, String> {
    let mut video_windows = state.video_windows.lock().unwrap();
    if let Some(window_label) = video_windows.get("current") {
        if let Some(window) = app.get_webview_window(window_label) {
            window.close().map_err(|e| format!("Failed to close video window: {}", e))?;
        }
        video_windows.remove("current");
    }
    
    // Clean up temporary files
    let temp_dir = std::env::temp_dir().join("wallpaper_manager");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
    
    Ok("Video wallpaper stopped and cleaned up".to_string())
}
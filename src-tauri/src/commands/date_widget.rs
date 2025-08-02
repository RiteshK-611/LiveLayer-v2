use tauri::{AppHandle, State, Wry, WebviewUrl};
use crate::state::AppState;
use crate::types::DateWidgetSettings;
use crate::commands::update_date_widget_state;
use tauri::Manager;
use serde_json;

#[tauri::command]
pub async fn create_date_widget(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
    settings: DateWidgetSettings,
) -> Result<String, String> {
    // Create unique window label for date widget
    let window_label = "date-widget";

    // Handle existing date widget
    {
        let mut date_widgets = state.date_widgets.lock().unwrap();
        if let Some(existing_label) = date_widgets.get("current") {
            if let Some(window) = app.get_webview_window(existing_label) {
                let _ = window.close();
            }
        }
        date_widgets.insert("current".to_string(), window_label.to_string());
    }

    // Serialize settings to pass to the widget
    let settings_json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    let encoded_settings = urlencoding::encode(&settings_json);

    // Create date widget window with settings
    let widget_url = format!("date-widget.html?settings={}", encoded_settings);

    let date_window = tauri::WebviewWindowBuilder::new(
        &app,
        window_label,
        WebviewUrl::App(widget_url.into()),
    )
    .title("Date Widget")
    .minimizable(false)
    .maximizable(false)
    .closable(false)
    .resizable(false)
    .decorations(false)
    .shadow(false)
    .visible(false)
    .skip_taskbar(true)
    .always_on_top(false)
    .transparent(true)
    .inner_size(400.0, 200.0)
    .position(100.0, 100.0)
    .build()
    .map_err(|e| format!("Failed to create date widget window: {}", e))?;

    // Show window after setup
    date_window.show()
        .map_err(|e| format!("Failed to show date widget: {}", e))?;

    // Wait for window to be ready
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Windows-specific: Set window to desktop level
    #[cfg(target_os = "windows")]
    {
        let date_window_clone = date_window.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            crate::platform::windows::set_widget_on_desktop(&date_window_clone)
        }).await;
        
        match result {
            Ok(Ok(_)) => {
                #[cfg(debug_assertions)]
                println!("Successfully positioned date widget on desktop");
            }
            Ok(Err(_e)) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to position date widget: {}", _e);
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to execute widget positioning task: {}", _e);
            }
        }
    }

    // Save date widget state
    let _ = update_date_widget_state(app.clone(), settings).await;

    Ok("Date widget created successfully".to_string())
}

#[tauri::command]
pub async fn hide_date_widget(state: State<'_, AppState>, app: AppHandle<Wry>) -> Result<String, String> {
    let date_widgets = state.date_widgets.lock().unwrap();
    if let Some(window_label) = date_widgets.get("current") {
        if let Some(window) = app.get_webview_window(window_label) {
            window.hide().map_err(|e| format!("Failed to hide date widget: {}", e))?;
        }
    }
    Ok("Date widget hidden".to_string())
}

#[tauri::command]
pub async fn show_date_widget(state: State<'_, AppState>, app: AppHandle<Wry>) -> Result<String, String> {
    let date_widgets = state.date_widgets.lock().unwrap();
    if let Some(window_label) = date_widgets.get("current") {
        if let Some(window) = app.get_webview_window(window_label) {
            window.show().map_err(|e| format!("Failed to show date widget: {}", e))?;
        }
    }
    Ok("Date widget shown".to_string())
}

#[tauri::command]
pub async fn close_date_widget(state: State<'_, AppState>, app: AppHandle<Wry>) -> Result<String, String> {
    let window_label = {
        let date_widgets = state.date_widgets.lock().unwrap();
        date_widgets.get("current").cloned()
    };
    
    if let Some(label) = window_label {
        if let Some(window) = app.get_webview_window(&label) {
            window.close().map_err(|e| format!("Failed to close date widget: {}", e))?;
        }
        let mut date_widgets = state.date_widgets.lock().unwrap();
        date_widgets.remove("current");
    }
    
    // Update state to reflect widget is disabled
    let current_state = crate::commands::load_app_state(app.clone()).await.unwrap_or_default();
    if let Some(mut widget_settings) = current_state.date_widget_settings {
        widget_settings.enabled = false;
        let _ = update_date_widget_state(app, widget_settings).await;
    }
    
    Ok("Date widget closed".to_string())
}

#[tauri::command]
pub async fn update_date_widget(
    state: State<'_, AppState>, 
    app: AppHandle<Wry>,
    settings: DateWidgetSettings
) -> Result<String, String> {
    // Close existing widget and create new one with updated settings
    let window_label = {
        let date_widgets = state.date_widgets.lock().unwrap();
        date_widgets.get("current").cloned()
    };
    
    if let Some(label) = window_label {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
    }
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    create_date_widget(app, state, settings).await
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperInfo {
    pub path: String,
    pub name: String,
    pub file_type: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateWidgetSettings {
    pub enabled: bool,
    pub locked: bool,
    pub show_time: bool,
    pub bold_text: bool,
    pub scale: f64,
    pub color: String,
    pub font: String,
    pub alignment: String,
    pub position_x: f64,
    pub position_y: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppPersistentState {
    pub last_wallpaper_path: Option<String>,
    pub last_wallpaper_file_type: Option<String>,
    pub date_widget_settings: Option<DateWidgetSettings>,
    pub wallpaper_settings: Option<WallpaperSettings>,
    pub wallpaper_list: Vec<WallpaperInfo>,
    pub autostart_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperSettings {
    pub auto_change: bool,
    pub interval: u32,
    pub random_order: bool,
    pub pause_on_fullscreen: bool,
}
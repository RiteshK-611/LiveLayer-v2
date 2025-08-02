export interface WallpaperInfo {
  path: string;
  name: string;
  file_type: string;
  size: number;
}

export interface WallpaperSettings {
  autoChange: boolean;
  interval: number;
  randomOrder: boolean;
  pauseOnFullscreen: boolean;
}

export interface DateWidgetSettings {
  enabled: boolean;
  locked: boolean;
  show_time: boolean;
  bold_text: boolean;
  scale: number;
  color: string;
  font: string;
  alignment: 'left' | 'center' | 'right';
  position_x: number;
  position_y: number;
}

export interface AppPersistentState {
  last_wallpaper_path?: string;
  last_wallpaper_file_type?: string;
  date_widget_settings?: DateWidgetSettings;
  wallpaper_settings?: WallpaperSettings;
  wallpaper_list: WallpaperInfo[];
  autostart_enabled: boolean;
}

export interface FontOption {
  name: string;
  value: string;
  type: 'google' | 'local';
  url?: string;
}
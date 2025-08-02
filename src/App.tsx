import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import WallpaperManager from "./components/WallpaperManager";
import DateWidget from "./components/DateWidget";
import { WallpaperSettings, DateWidgetSettings, AppPersistentState } from "./types/wallpaper";
import "./index.css";

function App() {
  const [activeTab, setActiveTab] = useState<"wallpaper" | "datewidget">(
    "wallpaper"
  );
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [loading, setLoading] = useState(true);
  const [wallpaperSettings, setWallpaperSettings] = useState<WallpaperSettings>(
    {
      autoChange: false,
      interval: 30,
      randomOrder: false,
      pauseOnFullscreen: true,
    }
  );

  const [dateWidgetSettings, setDateWidgetSettings] =
    useState<DateWidgetSettings>({
      enabled: false,
      locked: false,
      show_time: true,
      bold_text: false,
      scale: 1,
      color: "#FF",
      font: "Megrim",
      alignment: "center",
    });

  // Load persistent state on app startup
  useEffect(() => {
    const loadAppState = async () => {
      try {
        const state = await invoke<AppPersistentState>('load_app_state');
        
        // Load wallpaper settings
        if (state.wallpaper_settings) {
          setWallpaperSettings(state.wallpaper_settings);
        }
        
        // Load date widget settings
        if (state.date_widget_settings) {
          setDateWidgetSettings(state.date_widget_settings);
        }
        
        // Load autostart status
        const autostartStatus = await invoke<boolean>('get_autostart_status');
        setAutostartEnabled(autostartStatus);
        
      } catch (error) {
        console.error('Error loading app state:', error);
      } finally {
        setLoading(false);
      }
    };

    loadAppState();
  }, []);

  // Save wallpaper settings when they change
  useEffect(() => {
    if (!loading) {
      invoke('update_wallpaper_settings_state', { settings: wallpaperSettings })
        .catch(error => console.error('Error saving wallpaper settings:', error));
    }
  }, [wallpaperSettings, loading]);

  const handleAutostartToggle = async () => {
    try {
      await invoke('set_autostart', { enable: !autostartEnabled });
      setAutostartEnabled(!autostartEnabled);
    } catch (error) {
      console.error('Error toggling autostart:', error);
    }
  };

  const hideWindow = async () => {
    try {
      await invoke("hide_main_window");
    } catch (error) {
      console.error("Error hiding window:", error);
    }
  };

  if (loading) {
    return (
      <div className="app">
        <div className="loading-indicator" style={{ 
          height: '100vh', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center' 
        }}>
          <div className="spinner"></div>
          <span>Loading Wallora...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-left">
          <div style={{ display: "flex", alignItems: "center" }}>
            <img
              src="../src-tauri/icons/wallora.png"
              alt="Wallora Logo"
              style={{ width: "36px", height: "36px" }}
            />
            <h1>allora</h1>
          </div>
          {/* <p className="app-subtitle">Premium Wallpaper Experience</p> */}
        </div>
        <div className="header-actions">
          <label className="autostart-control">
            <input
              type="checkbox"
              checked={autostartEnabled}
              onChange={handleAutostartToggle}
            />
            <span>Auto-start</span>
          </label>
          <button onClick={hideWindow} className="btn btn-ghost">
            <span className="btn-icon">−</span>
            Minimize to Tray
          </button>
        </div>
      </header>

      <nav className="tab-navigation">
        <button
          className={`tab-button ${activeTab === "wallpaper" ? "active" : ""}`}
          onClick={() => setActiveTab("wallpaper")}>
          <span className="tab-icon">📷</span>
          Wallpaper Manager
        </button>
        <button
          className={`tab-button ${activeTab === "datewidget" ? "active" : ""}`}
          onClick={() => setActiveTab("datewidget")}>
          <span className="tab-icon">📆</span>
          Date Widget
        </button>
      </nav>

      <main className="main-content">
        {activeTab === "wallpaper" && (
          <WallpaperManager
            settings={wallpaperSettings}
            onSettingsChange={setWallpaperSettings}
          />
        )}
        {activeTab === "datewidget" && (
          <DateWidget
            settings={dateWidgetSettings}
            onSettingsChange={setDateWidgetSettings}
          />
        )}
      </main>
    </div>
  );
}

export default App;

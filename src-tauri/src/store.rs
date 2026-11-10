use crate::model::{Settings, State};
use crate::util::{atomic_write, expand};
use std::fs;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("umbriel-shaders")
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("umbriel-shaders")
}

pub fn library_dir(settings: &Settings) -> PathBuf {
    PathBuf::from(&settings.library_path)
}

pub fn generated_dir() -> PathBuf {
    data_dir().join("generated")
}

pub fn catalogue_file() -> PathBuf {
    data_dir().join("catalogue.json")
}

pub fn default_settings() -> Settings {
    let library = data_dir().join("library");
    let original = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("umbriel/config.toml");
    Settings {
        library_path: library.to_string_lossy().to_string(),
        original_config: original.to_string_lossy().to_string(),
        language: "es".into(),
        reduce_motion: false,
    }
}

pub fn load_settings() -> Settings {
    let path = config_dir().join("settings.json");
    let mut settings = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_else(default_settings);
    if settings.language != "es" && settings.language != "en" {
        settings.language = "es".into();
    }
    settings.library_path = expand(&settings.library_path);
    settings.original_config = expand(&settings.original_config);
    let _ = fs::create_dir_all(&settings.library_path);
    let _ = fs::create_dir_all(config_dir());
    settings
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    atomic_write(&config_dir().join("settings.json"), &body)
}

pub fn load_state() -> State {
    let path = config_dir().join("state.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<State>(&s).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &State) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    atomic_write(&config_dir().join("state.json"), &body)
}

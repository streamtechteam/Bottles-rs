use crate::error::{IoResultExt, Result};
use crate::paths::{Paths, APP_ID};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Exact GSettings keys from the Python schema contract.
pub const SETTING_KEYS: &[&str] = &[
    "flatpak-migration",
    "dark-theme",
    "ui-language",
    "force-offline",
    "update-date",
    "bottle-order",
    "font-scale",
    "steam-programs",
    "epic-games",
    "ubisoft-connect",
    "window-width",
    "window-height",
    "notifications",
    "show-component-updates",
    "eagle-security-scan",
    "eagle-crash-detection",
    "temp",
    "release-candidate",
    "startup-view",
    "experiments",
    "steam-proton-support",
    "auto-close-bottles",
    "audio-driver",
    "disable-home-drive",
    "show-sandbox-warning",
    "playtime-enabled",
    "playtime-heartbeat-interval",
    "eagle-scan-limit",
    "show-funding",
    "umu-launcher-path",
    "umu-data-path",
    "umu-proton",
    "umu-dependency-tool",
];

/// Typed mirror of the GSettings schema.  Values are read through the real
/// GSettings database when available and fall back to the schema defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub flatpak_migration: bool,
    pub dark_theme: bool,
    pub ui_language: String,
    pub force_offline: bool,
    pub update_date: bool,
    pub bottle_order: Vec<String>,
    pub font_scale: f64,
    pub steam_programs: bool,
    pub epic_games: bool,
    pub ubisoft_connect: bool,
    pub window_width: i64,
    pub window_height: i64,
    pub notifications: bool,
    pub show_component_updates: bool,
    pub eagle_security_scan: bool,
    pub eagle_crash_detection: bool,
    pub temp: bool,
    pub release_candidate: bool,
    pub startup_view: String,
    pub experiments: bool,
    pub steam_proton_support: bool,
    pub auto_close_bottles: bool,
    pub audio_driver: String,
    pub disable_home_drive: bool,
    pub show_sandbox_warning: bool,
    pub playtime_enabled: bool,
    pub playtime_heartbeat_interval: i64,
    pub eagle_scan_limit: i64,
    pub show_funding: bool,
    pub umu_launcher_path: String,
    pub umu_data_path: String,
    pub umu_proton: String,
    pub umu_dependency_tool: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            flatpak_migration: true,
            dark_theme: false,
            ui_language: "system".to_owned(),
            force_offline: false,
            update_date: false,
            bottle_order: Vec::new(),
            font_scale: 1.0,
            steam_programs: true,
            epic_games: true,
            ubisoft_connect: true,
            window_width: 880,
            window_height: 640,
            notifications: true,
            show_component_updates: true,
            eagle_security_scan: true,
            eagle_crash_detection: true,
            temp: false,
            release_candidate: false,
            startup_view: "page_library".to_owned(),
            experiments: false,
            steam_proton_support: false,
            auto_close_bottles: false,
            audio_driver: "default".to_owned(),
            disable_home_drive: false,
            show_sandbox_warning: true,
            playtime_enabled: true,
            playtime_heartbeat_interval: 60,
            eagle_scan_limit: 50,
            show_funding: true,
            umu_launcher_path: String::new(),
            umu_data_path: String::new(),
            umu_proton: "ProtoSoda".to_owned(),
            umu_dependency_tool: "bottles".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GSettingsStore {
    pub schema: String,
}

impl Default for GSettingsStore {
    fn default() -> Self {
        Self {
            schema: APP_ID.to_owned(),
        }
    }
}

impl GSettingsStore {
    pub fn get_raw(&self, key: &str) -> Option<String> {
        let output = Command::new("gsettings")
            .args(["get", &self.schema, key])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    pub fn set_raw(&self, key: &str, value: &str) -> Result<()> {
        let output = Command::new("gsettings")
            .args(["set", &self.schema, key, value])
            .output()
            .map_err(|source| crate::error::BottlesError::Io {
                path: PathBuf::from("gsettings"),
                source,
            })?;
        if !output.status.success() {
            return Err(crate::error::BottlesError::CommandFailed {
                command: format!("gsettings set {} {}", self.schema, key),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(())
    }
}

impl AppSettings {
    /// Load all schema keys from GSettings.  Missing schemas or values fall
    /// back to the exact defaults declared in the XML schema.
    pub fn load() -> Self {
        let store = GSettingsStore::default();
        let defaults = Self::default();
        Self {
            flatpak_migration: store
                .get_bool("flatpak-migration")
                .unwrap_or(defaults.flatpak_migration),
            dark_theme: store.get_bool("dark-theme").unwrap_or(defaults.dark_theme),
            ui_language: store
                .get_string("ui-language")
                .unwrap_or(defaults.ui_language),
            force_offline: store
                .get_bool("force-offline")
                .unwrap_or(defaults.force_offline),
            update_date: store
                .get_bool("update-date")
                .unwrap_or(defaults.update_date),
            bottle_order: store
                .get_strv("bottle-order")
                .unwrap_or(defaults.bottle_order),
            font_scale: store
                .get_double("font-scale")
                .unwrap_or(defaults.font_scale),
            steam_programs: store
                .get_bool("steam-programs")
                .unwrap_or(defaults.steam_programs),
            epic_games: store.get_bool("epic-games").unwrap_or(defaults.epic_games),
            ubisoft_connect: store
                .get_bool("ubisoft-connect")
                .unwrap_or(defaults.ubisoft_connect),
            window_width: store
                .get_int("window-width")
                .unwrap_or(defaults.window_width),
            window_height: store
                .get_int("window-height")
                .unwrap_or(defaults.window_height),
            notifications: store
                .get_bool("notifications")
                .unwrap_or(defaults.notifications),
            show_component_updates: store
                .get_bool("show-component-updates")
                .unwrap_or(defaults.show_component_updates),
            eagle_security_scan: store
                .get_bool("eagle-security-scan")
                .unwrap_or(defaults.eagle_security_scan),
            eagle_crash_detection: store
                .get_bool("eagle-crash-detection")
                .unwrap_or(defaults.eagle_crash_detection),
            temp: store.get_bool("temp").unwrap_or(defaults.temp),
            release_candidate: store
                .get_bool("release-candidate")
                .unwrap_or(defaults.release_candidate),
            startup_view: store
                .get_string("startup-view")
                .unwrap_or(defaults.startup_view),
            experiments: store
                .get_bool("experiments")
                .unwrap_or(defaults.experiments),
            steam_proton_support: store
                .get_bool("steam-proton-support")
                .unwrap_or(defaults.steam_proton_support),
            auto_close_bottles: store
                .get_bool("auto-close-bottles")
                .unwrap_or(defaults.auto_close_bottles),
            audio_driver: store
                .get_string("audio-driver")
                .unwrap_or(defaults.audio_driver),
            disable_home_drive: store
                .get_bool("disable-home-drive")
                .unwrap_or(defaults.disable_home_drive),
            show_sandbox_warning: store
                .get_bool("show-sandbox-warning")
                .unwrap_or(defaults.show_sandbox_warning),
            playtime_enabled: store
                .get_bool("playtime-enabled")
                .unwrap_or(defaults.playtime_enabled),
            playtime_heartbeat_interval: store
                .get_int("playtime-heartbeat-interval")
                .unwrap_or(defaults.playtime_heartbeat_interval),
            eagle_scan_limit: store
                .get_int("eagle-scan-limit")
                .unwrap_or(defaults.eagle_scan_limit),
            show_funding: store
                .get_bool("show-funding")
                .unwrap_or(defaults.show_funding),
            umu_launcher_path: store
                .get_string("umu-launcher-path")
                .unwrap_or(defaults.umu_launcher_path),
            umu_data_path: store
                .get_string("umu-data-path")
                .unwrap_or(defaults.umu_data_path),
            umu_proton: store
                .get_string("umu-proton")
                .unwrap_or(defaults.umu_proton),
            umu_dependency_tool: store
                .get_string("umu-dependency-tool")
                .unwrap_or(defaults.umu_dependency_tool),
        }
    }

    /// Persist the settings that the Rust front end can currently change.
    pub fn save(&self) -> Result<()> {
        let store = GSettingsStore::default();
        store.set_gvariant("dark-theme", &format_gvariant_bool(self.dark_theme))?;
        store.set_gvariant("font-scale", &format_gvariant_double(self.font_scale))?;
        store.set_gvariant("startup-view", &format_gvariant_string(&self.startup_view))?;
        store.set_gvariant("ui-language", &format_gvariant_string(&self.ui_language))?;
        store.set_gvariant("force-offline", &format_gvariant_bool(self.force_offline))?;
        store.set_gvariant("notifications", &format_gvariant_bool(self.notifications))?;
        store.set_gvariant(
            "playtime-enabled",
            &format_gvariant_bool(self.playtime_enabled),
        )?;
        store.set_gvariant("show-funding", &format_gvariant_bool(self.show_funding))?;
        store.set_gvariant("experiments", &format_gvariant_bool(self.experiments))?;
        store.set_gvariant(
            "release-candidate",
            &format_gvariant_bool(self.release_candidate),
        )?;
        Ok(())
    }
}

impl GSettingsStore {
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        parse_gvariant_bool(&self.get_raw(key)?)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        parse_gvariant_string(&self.get_raw(key)?)
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_raw(key)?.parse::<i64>().ok()
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        self.get_raw(key)?.parse::<f64>().ok()
    }

    pub fn get_strv(&self, key: &str) -> Option<Vec<String>> {
        parse_gvariant_strv(&self.get_raw(key)?)
    }

    pub fn set_gvariant(&self, key: &str, value: &str) -> Result<()> {
        self.set_raw(key, value)
    }
}

pub fn parse_gvariant_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn parse_gvariant_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"')))
    {
        let quote = value.chars().next()?;
        let inner = &value[1..value.len() - 1];
        let mut result = String::new();
        let mut escaped = false;
        for character in inner.chars() {
            if escaped {
                result.push(match character {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => other,
                });
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                return None;
            } else {
                result.push(character);
            }
        }
        Some(result)
    } else {
        Some(value.to_owned())
    }
}

pub fn parse_gvariant_strv(value: &str) -> Option<Vec<String>> {
    let value = value.trim();
    if value == "@as []" || value == "[]" {
        return Some(Vec::new());
    }
    if !(value.starts_with('[') && value.ends_with(']')) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for character in inner.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
            current.push(character);
        } else if let Some(active) = quote {
            if character == active {
                quote = None;
            }
            current.push(character);
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            current.push(character);
        } else if character == ',' {
            items.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(character);
        }
    }
    if !current.trim().is_empty() {
        items.push(current.trim().to_owned());
    }
    items
        .into_iter()
        .map(|item| parse_gvariant_string(&item))
        .collect()
}

pub fn format_gvariant_bool(value: bool) -> String {
    value.to_string()
}

pub fn format_gvariant_double(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

pub fn format_gvariant_string(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// User data stored in `data.yml`, mirroring `DataManager` and `UserDataKeys`.
#[derive(Debug, Clone)]
pub struct UserData {
    pub path: PathBuf,
    pub values: BTreeMap<String, Value>,
}

pub struct UserDataKeys;

impl UserDataKeys {
    pub const CUSTOM_BOTTLES_PATH: &'static str = "custom_bottles_path";
    pub const EAGLE_INTEL_ANNOUNCEMENT_SEEN: &'static str = "eagle_intel_announcement_seen";
    pub const FUNDING_DISMISSED: &'static str = "funding_dialog_dismissed_next";
    pub const LAST_FUNDING_PROMPT: &'static str = "last_funding_prompt_date";
    pub const LAST_FUNDING_MAJOR: &'static str = "last_funding_major";
    pub const FUNDING_PROMPT_COUNT: &'static str = "funding_prompt_count";
    pub const FUNDING_SUPPORTER: &'static str = "funding_supporter";
    pub const PERSONAL_REPOSITORIES: &'static str = "personal_repositories";
    pub const TRUSTED_EXECUTABLES: &'static str = "trusted_executables";
}

impl UserData {
    pub fn load(paths: &Paths) -> Result<Self> {
        let mut result = Self {
            path: paths.data.clone(),
            values: Self::default_values(),
        };
        match fs::read_to_string(&result.path) {
            Ok(content) => {
                let value: Value = serde_yaml::from_str(&content)?;
                if let Value::Mapping(mapping) = value {
                    for (key, value) in mapping {
                        if let Some(key) = key.as_str() {
                            result.values.insert(key.to_owned(), value);
                        }
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => result.save()?,
            Err(error) => {
                return Err(crate::error::BottlesError::Io {
                    path: result.path.clone(),
                    source: error,
                })
            }
        }
        Ok(result)
    }

    fn default_values() -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("funding_dialog_dismissed".to_owned(), Value::Bool(false)),
            (
                UserDataKeys::PERSONAL_REPOSITORIES.to_owned(),
                Value::Mapping(serde_yaml::Mapping::new()),
            ),
        ])
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(Value::as_bool).unwrap_or(default)
    }

    pub fn get_string(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_owned()
    }

    /// Match Python's `DataManager.set`: an existing sequence receives an
    /// appended value; every other truthy value is replaced.
    pub fn set(&mut self, key: &str, value: Value, as_list: bool) -> Result<()> {
        match self.values.get_mut(key) {
            Some(Value::Sequence(sequence)) => sequence.push(value),
            Some(existing) => *existing = value,
            None if as_list => {
                self.values
                    .insert(key.to_owned(), Value::Sequence(vec![value]));
            }
            None => {
                self.values.insert(key.to_owned(), value);
            }
        }
        self.save()
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        self.values.remove(key);
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        fs::create_dir_all(parent).with_path(parent)?;
        let content = serde_yaml::to_string(&self.values)?;
        fs::write(&self.path, content).with_path(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_keys_match_python_schema() {
        let contract = crate::parity::contracts().settings.keys.clone();
        let actual: Vec<String> = contract.iter().map(|key| key.name.clone()).collect();
        assert_eq!(actual, SETTING_KEYS);
    }

    #[test]
    fn defaults_match_gsettings_schema() {
        let settings = AppSettings::default();
        assert_eq!(settings.window_width, 880);
        assert_eq!(settings.window_height, 640);
        assert_eq!(settings.startup_view, "page_library");
        assert_eq!(settings.audio_driver, "default");
        assert_eq!(settings.umu_proton, "ProtoSoda");
        assert_eq!(settings.playtime_heartbeat_interval, 60);
    }

    #[test]
    fn gvariant_parsing_round_trips() {
        assert_eq!(parse_gvariant_string("'system'"), Some("system".to_owned()));
        assert_eq!(parse_gvariant_string("''"), Some(String::new()));
        assert_eq!(parse_gvariant_bool("true"), Some(true));
        assert_eq!(format_gvariant_double(1.0), "1.0");
        let value = parse_gvariant_strv("['a', 'b']").unwrap();
        assert_eq!(value, vec!["a".to_owned(), "b".to_owned()]);
    }
}

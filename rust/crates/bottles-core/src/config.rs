use crate::error::{BottlesError, IoResultExt, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Exact `BottleConfig` field names from the generated Python contract.
pub const BOTTLE_CONFIG_FIELDS: &[&str] = &[
    "Name",
    "Arch",
    "Windows",
    "Runner",
    "WorkingDir",
    "D7VK",
    "DXVK",
    "NVAPI",
    "VKD3D",
    "LatencyFleX",
    "Path",
    "Custom_Path",
    "Environment",
    "Creation_Date",
    "Update_Date",
    "Versioning",
    "Versioning_Exclusion_Patterns",
    "State",
    "Parameters",
    "Sandbox",
    "Limit_System_Environment",
    "Environment_Variables",
    "Inherited_Environment_Variables",
    "Installed_Dependencies",
    "DLL_Overrides",
    "External_Programs",
    "Uninstallers",
    "Registry_Rules",
    "session_arguments",
    "run_in_terminal",
    "Language",
    "Winebridge",
    "CompatData",
    "data",
    "RunnerPath",
];

/// Exact `BottleParams` field names from the generated Python contract.
pub const BOTTLE_PARAMS_FIELDS: &[&str] = &[
    "d7vk",
    "dxvk",
    "dxvk_nvapi",
    "vkd3d",
    "latencyflex",
    "mangohud",
    "mangohud_display_on_game_start",
    "obsvkc",
    "vkbasalt",
    "lsfg_vk",
    "lsfg_vk_multiplier",
    "lsfg_vk_flow_scale",
    "lsfg_vk_performance_mode",
    "gamemode",
    "gamescope",
    "gamescope_game_width",
    "gamescope_game_height",
    "gamescope_window_width",
    "gamescope_window_height",
    "gamescope_fps",
    "gamescope_fps_no_focus",
    "gamescope_scaling",
    "gamescope_borderless",
    "gamescope_fullscreen",
    "gamescope_custom_options",
    "sync",
    "frame_rate_limit",
    "hidraw_devices",
    "fsr",
    "fsr_sharpening_strength",
    "custom_dpi",
    "renderer",
    "discrete_gpu",
    "virtual_desktop",
    "virtual_desktop_res",
    "wayland",
    "hdr",
    "pulseaudio_latency",
    "fullscreen_capture",
    "take_focus",
    "mouse_warp",
    "decorated",
    "fixme_logs",
    "use_runtime",
    "use_eac_runtime",
    "use_be_runtime",
    "use_steam_runtime",
    "winebridge",
    "sandbox",
    "versioning_compression",
    "versioning_automatic",
    "versioning_exclusion_patterns",
    "vmtouch",
    "vmtouch_cache_cwd",
    "adaptive_launch",
    "eagle_tracing",
];

pub const BOTTLE_SANDBOX_PARAMS_FIELDS: &[&str] =
    &["share_net", "share_sound", "share_input", "share_usb"];

/// Default inherited environment list from `Samples.default_inherited_environment`.
pub const DEFAULT_INHERITED_ENVIRONMENT: &[&str] = &[
    "DBUS_SESSION_BUS_ADDRESS",
    "DISPLAY",
    "HOME",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "LC_MESSAGES",
    "MANGOHUD_CONFIG",
    "PATH",
    "PULSE_SERVER",
    "TERM",
    "TZ",
    "USER",
    "WAYLAND_DISPLAY",
    "XAUTHORITY",
    "XDG_CONFIG_HOME",
    "XDG_RUNTIME_DIR",
    "XMODIFIERS",
    "XCURSOR_SIZE",
];

/// The Python model stores either the empty-string default or a boolean in
/// `Custom_Path`.  Older configurations in the wild contain both forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomPath {
    Bool(bool),
    Text(String),
}

impl Default for CustomPath {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl CustomPath {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Text(value) => !value.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BottleSandboxParams {
    pub share_net: bool,
    pub share_sound: bool,
    pub share_input: bool,
    pub share_usb: bool,
}

impl Default for BottleSandboxParams {
    fn default() -> Self {
        Self {
            share_net: false,
            share_sound: false,
            share_input: false,
            share_usb: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BottleParams {
    pub d7vk: bool,
    pub dxvk: bool,
    pub dxvk_nvapi: bool,
    pub vkd3d: bool,
    pub latencyflex: bool,
    pub mangohud: bool,
    pub mangohud_display_on_game_start: bool,
    pub obsvkc: bool,
    pub vkbasalt: bool,
    pub lsfg_vk: bool,
    pub lsfg_vk_multiplier: i64,
    pub lsfg_vk_flow_scale: f64,
    pub lsfg_vk_performance_mode: bool,
    pub gamemode: bool,
    pub gamescope: bool,
    pub gamescope_game_width: i64,
    pub gamescope_game_height: i64,
    pub gamescope_window_width: i64,
    pub gamescope_window_height: i64,
    pub gamescope_fps: i64,
    pub gamescope_fps_no_focus: i64,
    pub gamescope_scaling: bool,
    pub gamescope_borderless: bool,
    pub gamescope_fullscreen: bool,
    pub gamescope_custom_options: String,
    pub sync: String,
    pub frame_rate_limit: i64,
    pub hidraw_devices: Vec<String>,
    pub fsr: bool,
    pub fsr_sharpening_strength: i64,
    pub custom_dpi: i64,
    pub renderer: String,
    pub discrete_gpu: bool,
    pub virtual_desktop: bool,
    pub virtual_desktop_res: String,
    pub wayland: bool,
    pub hdr: bool,
    pub pulseaudio_latency: bool,
    pub fullscreen_capture: bool,
    pub take_focus: bool,
    pub mouse_warp: bool,
    pub decorated: bool,
    pub fixme_logs: bool,
    pub use_runtime: bool,
    pub use_eac_runtime: bool,
    pub use_be_runtime: bool,
    pub use_steam_runtime: bool,
    pub winebridge: bool,
    pub sandbox: bool,
    pub versioning_compression: bool,
    pub versioning_automatic: bool,
    pub versioning_exclusion_patterns: bool,
    pub vmtouch: bool,
    pub vmtouch_cache_cwd: bool,
    pub adaptive_launch: bool,
    pub eagle_tracing: bool,
}

impl Default for BottleParams {
    fn default() -> Self {
        Self {
            d7vk: false,
            dxvk: false,
            dxvk_nvapi: false,
            vkd3d: false,
            latencyflex: false,
            mangohud: false,
            mangohud_display_on_game_start: true,
            obsvkc: false,
            vkbasalt: false,
            lsfg_vk: false,
            lsfg_vk_multiplier: 2,
            lsfg_vk_flow_scale: 1.0,
            lsfg_vk_performance_mode: false,
            gamemode: false,
            gamescope: false,
            gamescope_game_width: 0,
            gamescope_game_height: 0,
            gamescope_window_width: 0,
            gamescope_window_height: 0,
            gamescope_fps: 0,
            gamescope_fps_no_focus: 0,
            gamescope_scaling: false,
            gamescope_borderless: false,
            gamescope_fullscreen: true,
            gamescope_custom_options: String::new(),
            sync: "wine".to_owned(),
            frame_rate_limit: 0,
            hidraw_devices: Vec::new(),
            fsr: false,
            fsr_sharpening_strength: 8,
            custom_dpi: 96,
            renderer: "gl".to_owned(),
            discrete_gpu: false,
            virtual_desktop: false,
            virtual_desktop_res: "1280x720".to_owned(),
            wayland: false,
            hdr: false,
            pulseaudio_latency: false,
            fullscreen_capture: false,
            take_focus: false,
            mouse_warp: true,
            decorated: true,
            fixme_logs: false,
            use_runtime: false,
            use_eac_runtime: true,
            use_be_runtime: true,
            use_steam_runtime: false,
            winebridge: false,
            sandbox: false,
            versioning_compression: false,
            versioning_automatic: false,
            versioning_exclusion_patterns: false,
            vmtouch: false,
            vmtouch_cache_cwd: false,
            adaptive_launch: false,
            eagle_tracing: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExternalProgram {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_script_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_script_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d7vk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vkd3d: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk_nvapi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamescope: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_desktop: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winebridge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_console: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autostart: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discrete_gpu: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamemode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latencyflex: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_backup: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub monitoring: Vec<String>,
    /// Preserve program fields introduced by newer Python versions.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl ExternalProgram {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.executable.clone())
            .or_else(|| self.id.clone())
            .unwrap_or_else(|| "Unnamed program".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RegistryRule {
    pub name: String,
    pub description: String,
    pub keys: String,
    pub triggers: Vec<String>,
    pub run_once: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(default)]
pub struct EnvironmentTemplate {
    #[serde(rename = "Runner")]
    pub runner: Option<String>,
    #[serde(rename = "Parameters")]
    pub parameters: BTreeMap<String, Value>,
    #[serde(rename = "Installed_Dependencies")]
    pub installed_dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BottleConfig {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Arch")]
    pub arch: String,
    #[serde(rename = "Windows")]
    pub windows: String,
    #[serde(rename = "Runner")]
    pub runner: String,
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,
    #[serde(rename = "D7VK")]
    pub d7vk: String,
    #[serde(rename = "DXVK")]
    pub dxvk: String,
    #[serde(rename = "NVAPI")]
    pub nvapi: String,
    #[serde(rename = "VKD3D")]
    pub vkd3d: String,
    #[serde(rename = "LatencyFleX")]
    pub latency_flex: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Custom_Path")]
    pub custom_path: CustomPath,
    #[serde(rename = "Environment")]
    pub environment: String,
    #[serde(rename = "Creation_Date")]
    pub creation_date: String,
    #[serde(rename = "Update_Date")]
    pub update_date: String,
    #[serde(rename = "Versioning")]
    pub versioning: bool,
    #[serde(rename = "Versioning_Exclusion_Patterns")]
    pub versioning_exclusion_patterns: Vec<String>,
    #[serde(rename = "State")]
    pub state: i64,
    #[serde(rename = "Parameters")]
    pub parameters: BottleParams,
    #[serde(rename = "Sandbox")]
    pub sandbox: BottleSandboxParams,
    #[serde(rename = "Limit_System_Environment")]
    pub limit_system_environment: bool,
    #[serde(rename = "Environment_Variables")]
    pub environment_variables: BTreeMap<String, Value>,
    #[serde(rename = "Inherited_Environment_Variables")]
    pub inherited_environment_variables: Vec<String>,
    #[serde(rename = "Installed_Dependencies")]
    pub installed_dependencies: Vec<String>,
    #[serde(rename = "DLL_Overrides")]
    pub dll_overrides: BTreeMap<String, String>,
    #[serde(rename = "External_Programs")]
    pub external_programs: BTreeMap<String, ExternalProgram>,
    #[serde(rename = "Uninstallers")]
    pub uninstallers: BTreeMap<String, Value>,
    #[serde(rename = "Registry_Rules")]
    pub registry_rules: Vec<RegistryRule>,
    pub session_arguments: String,
    pub run_in_terminal: bool,
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "Winebridge")]
    pub winebridge: bool,
    #[serde(rename = "CompatData")]
    pub compat_data: String,
    pub data: BTreeMap<String, Value>,
    #[serde(rename = "RunnerPath")]
    pub runner_path: String,
}

impl Default for BottleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            arch: "win64".to_owned(),
            windows: "win10".to_owned(),
            runner: String::new(),
            working_dir: String::new(),
            d7vk: String::new(),
            dxvk: String::new(),
            nvapi: String::new(),
            vkd3d: String::new(),
            latency_flex: String::new(),
            path: String::new(),
            custom_path: CustomPath::default(),
            environment: String::new(),
            creation_date: String::new(),
            update_date: String::new(),
            versioning: false,
            versioning_exclusion_patterns: Vec::new(),
            state: 0,
            parameters: BottleParams::default(),
            sandbox: BottleSandboxParams::default(),
            limit_system_environment: false,
            environment_variables: BTreeMap::new(),
            inherited_environment_variables: Vec::new(),
            installed_dependencies: Vec::new(),
            dll_overrides: BTreeMap::new(),
            external_programs: BTreeMap::new(),
            uninstallers: BTreeMap::new(),
            registry_rules: Vec::new(),
            session_arguments: String::new(),
            run_in_terminal: false,
            language: "sys".to_owned(),
            winebridge: false,
            compat_data: String::new(),
            data: BTreeMap::new(),
            runner_path: String::new(),
        }
    }
}

impl BottleConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).with_path(path)?;
        Self::from_yaml_str(&content)
    }

    pub fn from_yaml_str(content: &str) -> Result<Self> {
        let mut value: Value = serde_yaml::from_str(content)?;
        migrate_config_value(&mut value)?;
        Ok(serde_yaml::from_value(value)?)
    }

    /// Like the Python `dump` method: write a temporary file in the destination
    /// directory, flush it, and atomically replace the destination.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).with_path(parent)?;
        let temp = parent.join(format!(".bottle-{}.tmp", uuid::Uuid::new_v4()));
        let content = serde_yaml::to_string(self)?;
        let mut file = File::create(&temp).with_path(&temp)?;
        file.write_all(content.as_bytes()).with_path(&temp)?;
        file.sync_all().with_path(&temp)?;
        match fs::rename(&temp, path) {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = fs::remove_file(&temp);
                Err(BottlesError::Io {
                    path: path.to_path_buf(),
                    source: error,
                })
            }
        }
    }

    pub fn to_yaml_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Generic version of `Manager.update_config` for the YAML scopes used by
    /// the Python CLI and UI.
    pub fn update_value(
        &mut self,
        scope: &str,
        key: &str,
        value: Option<Value>,
        remove: bool,
    ) -> Result<()> {
        let mut root = serde_yaml::to_value(&*self)?;
        let mapping = root
            .as_mapping_mut()
            .ok_or_else(|| BottlesError::Config("config root is not a mapping".to_owned()))?;
        let target = if scope.is_empty() {
            mapping
        } else {
            let key_value = Value::String(scope.to_owned());
            if !mapping.contains_key(&key_value) {
                mapping.insert(key_value.clone(), Value::Mapping(Mapping::new()));
            }
            mapping
                .get_mut(&key_value)
                .and_then(Value::as_mapping_mut)
                .ok_or_else(|| BottlesError::Config(format!("scope `{scope}` is not a mapping")))?
        };
        let field = Value::String(key.to_owned());
        if remove {
            target.remove(&field);
        } else {
            let value = value.ok_or_else(|| {
                BottlesError::Config("a value is required unless remove is true".to_owned())
            })?;
            target.insert(field, value);
        }
        *self = serde_yaml::from_value(root)?;
        Ok(())
    }

    pub fn set_environment_variable(&mut self, key: &str, value: &str) -> Result<()> {
        self.update_value(
            "Environment_Variables",
            key,
            Some(Value::String(value.to_owned())),
            false,
        )
    }

    pub fn set_parameter_from_text(&mut self, key: &str, value: &str) -> Result<()> {
        if !BOTTLE_PARAMS_FIELDS.contains(&key) {
            return Err(BottlesError::InvalidInput(format!(
                "Invalid parameter {key}"
            )));
        }
        let parsed = parse_parameter_value(value);
        self.update_value("Parameters", key, Some(parsed), false)
    }

    pub fn apply_environment_template(&mut self, template: EnvironmentTemplate) -> Result<()> {
        if let Some(runner) = template.runner {
            if self.runner.is_empty() || self.runner == "wine" {
                self.runner = runner;
            }
        }
        for (key, value) in template.parameters {
            self.update_value("Parameters", &key, Some(value), false)?;
        }
        if !template.installed_dependencies.is_empty() {
            self.installed_dependencies = template.installed_dependencies;
        }
        Ok(())
    }

    pub fn programs(&self) -> Vec<ExternalProgram> {
        let mut programs: Vec<ExternalProgram> = self
            .external_programs
            .values()
            .filter(|program| program.removed != Some(true))
            .cloned()
            .collect();
        programs.sort_by(|left, right| left.display_name().cmp(&right.display_name()));
        programs
    }
}

pub fn parse_parameter_value(value: &str) -> Value {
    match value.to_ascii_lowercase().as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" | "none" => Value::Null,
        _ => {
            if let Ok(number) = value.parse::<i64>() {
                Value::Number(number.into())
            } else if let Ok(number) = value.parse::<f64>() {
                Value::Number(serde_yaml::Number::from(number))
            } else {
                Value::String(value.to_owned())
            }
        }
    }
}

fn legacy_inherited_environments() -> Vec<Vec<&'static str>> {
    let legacy_base: Vec<&'static str> = vec![
        "DBUS_SESSION_BUS_ADDRESS",
        "DISPLAY",
        "HOME",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "LC_MESSAGES",
        "PATH",
        "PULSE_SERVER",
        "TERM",
        "TZ",
        "USER",
        "WAYLAND_DISPLAY",
        "XAUTHORITY",
        "XDG_RUNTIME_DIR",
    ];
    let mut full = legacy_base.clone();
    full.insert(7, "MANGOHUD_CONFIG");
    let mut result = vec![legacy_base.clone(), full.clone()];
    for base in [legacy_base, full] {
        let mut with_modifiers = base.clone();
        with_modifiers.push("XMODIFIERS");
        result.push(with_modifiers);
        let mut with_cursor = base.clone();
        with_cursor.push("XMODIFIERS");
        with_cursor.push("XCURSOR_SIZE");
        result.push(with_cursor);
    }
    result
}

fn sequence_as_strings(value: &Value) -> Option<Vec<String>> {
    let sequence = value.as_sequence()?;
    sequence
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn migrate_config_value(value: &mut Value) -> Result<()> {
    let root = value
        .as_mapping_mut()
        .ok_or_else(|| BottlesError::Config("bottle.yml must contain a mapping".to_owned()))?;

    for key in ["Parameters", "Sandbox"] {
        let field = Value::String(key.to_owned());
        if !matches!(root.get(&field), Some(Value::Mapping(_))) {
            root.insert(field, Value::Mapping(Mapping::new()));
        }
    }

    let limit = root
        .get(&Value::String("Limit_System_Environment".to_owned()))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let inherited_key = Value::String("Inherited_Environment_Variables".to_owned());
    if limit {
        if let Some(inherited) = root.get(&inherited_key).and_then(sequence_as_strings) {
            if legacy_inherited_environments().iter().any(|candidate| {
                candidate
                    .iter()
                    .map(|item| (*item).to_owned())
                    .collect::<Vec<_>>()
                    == inherited
            }) {
                let mut updated = inherited;
                for name in ["XMODIFIERS", "XDG_CONFIG_HOME"] {
                    if !updated.iter().any(|item| item == name) {
                        updated.push(name.to_owned());
                    }
                }
                root.insert(
                    inherited_key,
                    Value::Sequence(updated.into_iter().map(Value::String).collect()),
                );
            }
        }
    }

    let parameters_key = Value::String("Parameters".to_owned());
    if let Some(parameters) = root
        .get_mut(&parameters_key)
        .and_then(Value::as_mapping_mut)
    {
        let fsr_level = Value::String("fsr_level".to_owned());
        if let Some(value) = parameters.remove(&fsr_level) {
            parameters.insert(Value::String("fsr_sharpening_strength".to_owned()), value);
        }
        parameters.remove(&Value::String("show_component_updates".to_owned()));
    }

    for (old, new) in [("DXVK_NVAPI", "NVAPI"), ("LatencyFlex", "LatencyFleX")] {
        let old_key = Value::String(old.to_owned());
        if let Some(value) = root.remove(&old_key) {
            root.insert(Value::String(new.to_owned()), value);
        }
    }
    Ok(())
}

pub fn json_value_to_yaml(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(*value),
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Value::Number(value.into())
            } else if let Some(value) = number.as_f64() {
                Value::Number(serde_yaml::Number::from(value))
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(value) => Value::String(value.clone()),
        serde_json::Value::Array(values) => {
            Value::Sequence(values.iter().map(json_value_to_yaml).collect())
        }
        serde_json::Value::Object(values) => Value::Mapping(
            values
                .iter()
                .map(|(key, value)| (Value::String(key.clone()), json_value_to_yaml(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity::contracts;

    #[test]
    fn field_lists_match_python_contract() {
        let contract = &contracts().config_fields;
        let actual: Vec<String> = contract
            .bottle_config
            .iter()
            .map(|field| field.name.clone())
            .collect();
        assert_eq!(actual, BOTTLE_CONFIG_FIELDS);
        let actual: Vec<String> = contract
            .bottle_params
            .iter()
            .map(|field| field.name.clone())
            .collect();
        assert_eq!(actual, BOTTLE_PARAMS_FIELDS);
        let actual: Vec<String> = contract
            .bottle_sandbox_params
            .iter()
            .map(|field| field.name.clone())
            .collect();
        assert_eq!(actual, BOTTLE_SANDBOX_PARAMS_FIELDS);
    }

    #[test]
    fn defaults_match_python_sample() {
        let config = BottleConfig::default();
        assert_eq!(config.arch, "win64");
        assert_eq!(config.windows, "win10");
        assert_eq!(config.language, "sys");
        assert_eq!(config.parameters.sync, "wine");
        assert!(config.parameters.mouse_warp);
        assert!(config.parameters.decorated);
        assert!(config.parameters.gamescope_fullscreen);
        assert_eq!(config.parameters.fsr_sharpening_strength, 8);
        assert_eq!(config.parameters.custom_dpi, 96);
        assert_eq!(config.parameters.virtual_desktop_res, "1280x720");
    }

    #[test]
    fn legacy_config_migrations_match_python() {
        let yaml = r#"
Name: Legacy
Limit_System_Environment: true
Inherited_Environment_Variables:
  - DISPLAY
  - HOME
  - LANG
Parameters:
  fsr_level: 4
  show_component_updates: false
  dxvk: true
DXVK_NVAPI: old-nvapi
LatencyFlex: old-latencyflex
Unexpected: ignored
"#;
        let config = BottleConfig::from_yaml_str(yaml).unwrap();
        assert_eq!(config.name, "Legacy");
        assert_eq!(config.parameters.fsr_sharpening_strength, 4);
        assert_eq!(config.nvapi, "old-nvapi");
        assert_eq!(config.latency_flex, "old-latencyflex");
        assert!(config.parameters.dxvk);
        let serialized = config.to_yaml_string().unwrap();
        assert!(!serialized.contains("fsr_level:"));
        assert!(!serialized.contains("show_component_updates:"));
        assert!(!serialized.contains("Unexpected:"));
    }

    #[test]
    fn inherited_environment_migration_adds_only_missing_defaults() {
        let mut legacy: Vec<&str> = DEFAULT_INHERITED_ENVIRONMENT
            .iter()
            .copied()
            .filter(|item| {
                *item != "XDG_CONFIG_HOME" && *item != "XCURSOR_SIZE" && *item != "XMODIFIERS"
            })
            .collect();
        // The Python legacy list also omitted XDG_CONFIG_HOME.
        legacy.retain(|item| *item != "XDG_CONFIG_HOME");
        let mut root = serde_yaml::from_str::<Value>(
            "Name: Legacy\nLimit_System_Environment: true\nParameters: {}\nSandbox: {}\n",
        )
        .unwrap();
        root.as_mapping_mut().unwrap().insert(
            Value::String("Inherited_Environment_Variables".to_owned()),
            Value::Sequence(
                legacy
                    .iter()
                    .map(|item| Value::String((*item).to_owned()))
                    .collect(),
            ),
        );
        migrate_config_value(&mut root).unwrap();
        let config: BottleConfig = serde_yaml::from_value(root).unwrap();
        assert!(config
            .inherited_environment_variables
            .contains(&"XDG_CONFIG_HOME".to_owned()));
        assert!(config
            .inherited_environment_variables
            .contains(&"XMODIFIERS".to_owned()));
    }

    #[test]
    fn custom_path_accepts_legacy_string_and_current_bool() {
        let old: BottleConfig = serde_yaml::from_str("Custom_Path: ''").unwrap();
        let current: BottleConfig = serde_yaml::from_str("Custom_Path: true").unwrap();
        assert!(!old.custom_path.is_truthy());
        assert!(current.custom_path.is_truthy());
    }

    #[test]
    fn update_value_preserves_other_fields() {
        let mut config = BottleConfig::default();
        config.name = "Test".to_owned();
        config
            .update_value("Parameters", "dxvk", Some(Value::Bool(true)), false)
            .unwrap();
        config
            .set_environment_variable("WINEDEBUG", "-all")
            .unwrap();
        assert_eq!(config.name, "Test");
        assert!(config.parameters.dxvk);
        assert_eq!(
            config.environment_variables.get("WINEDEBUG"),
            Some(&Value::String("-all".to_owned()))
        );
    }
}

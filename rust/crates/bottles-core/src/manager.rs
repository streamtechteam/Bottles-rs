use crate::config::{
    BottleConfig, CustomPath, EnvironmentTemplate, ExternalProgram, DEFAULT_INHERITED_ENVIRONMENT,
};
use crate::error::{BottlesError, IoResultExt, Result};
use crate::paths::{sanitize_universal_filename, validate_bottle_name, ComponentKind, LegacyPaths, Paths};
use crate::wine::{WineLauncher, WineTool};
use chrono::Local;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bottle {
    pub directory_name: String,
    pub config_path: PathBuf,
    pub prefix: PathBuf,
    pub placeholder: Option<PathBuf>,
    pub config: BottleConfig,
}

impl Bottle {
    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn config_file(&self) -> PathBuf {
        self.prefix.join("bottle.yml")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewBottleOptions {
    pub name: String,
    pub environment: String,
    pub custom_environment: Option<PathBuf>,
    pub arch: String,
    pub path: Option<PathBuf>,
    pub runner: Option<String>,
    pub d7vk: Option<String>,
    pub dxvk: Option<String>,
    pub vkd3d: Option<String>,
    pub nvapi: Option<String>,
    pub latencyflex: Option<String>,
    pub versioning: bool,
    pub sandbox: bool,
}

impl Default for NewBottleOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            environment: "application".to_owned(),
            custom_environment: None,
            arch: "win64".to_owned(),
            path: None,
            runner: None,
            d7vk: None,
            dxvk: None,
            vkd3d: None,
            nvapi: None,
            latencyflex: None,
            versioning: false,
            sandbox: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinePrefixCandidate {
    pub name: String,
    pub manager: String,
    pub path: PathBuf,
    pub lock: bool,
}

#[derive(Debug, Clone)]
pub struct BottleManager {
    pub paths: Paths,
}

impl BottleManager {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn initialize(&self) -> Result<()> {
        self.paths.ensure_app_dirs()
    }

    /// Mirror `Manager.check_bottles`, including custom-path placeholders.
    pub fn list_bottles(&self) -> Result<Vec<Bottle>> {
        self.paths.ensure_app_dirs()?;
        let mut bottles = Vec::new();
        let mut entries = fs::read_dir(&self.paths.bottles)
            .with_path(&self.paths.bottles)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        entries.sort();
        for directory_name in entries {
            let local = self.paths.bottles.join(&directory_name);
            if !local.is_dir() {
                continue;
            }
            let placeholder = local.join("placeholder.yml");
            let mut placeholder_target = None;
            let config_path = if placeholder.is_file() {
                let content = fs::read_to_string(&placeholder).with_path(&placeholder)?;
                let value: Value = serde_yaml::from_str(&content)?;
                let target = value
                    .as_mapping()
                    .and_then(|mapping| mapping.get(&Value::String("Path".to_owned())))
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                match target {
                    Some(target) => {
                        placeholder_target = Some(target.clone());
                        target.join("bottle.yml")
                    }
                    None => continue,
                }
            } else {
                local.join("bottle.yml")
            };
            if !config_path.is_file() {
                continue;
            }
            let config = match BottleConfig::load(&config_path) {
                Ok(config) => config,
                Err(error) => {
                    log::warn!("Skipping broken bottle {directory_name}: {error}");
                    continue;
                }
            };
            let prefix = if placeholder_target.is_some() {
                config_path
                    .parent()
                    .unwrap_or(&self.paths.bottles)
                    .to_path_buf()
            } else {
                bottle_prefix_path(&self.paths, &config)
            };
            bottles.push(Bottle {
                directory_name,
                config_path,
                prefix,
                placeholder: placeholder.is_file().then_some(placeholder),
                config,
            });
        }
        Ok(bottles)
    }

    pub fn get_bottle(&self, name: &str) -> Result<Bottle> {
        self.list_bottles()?
            .into_iter()
            .find(|bottle| bottle.name() == name)
            .ok_or_else(|| BottlesError::BottleNotFound(name.to_owned()))
    }

    pub fn save_config(&self, bottle: &Bottle) -> Result<()> {
        bottle.config.save(&bottle.config_path)
    }

    pub fn update_config(&self, bottle: &mut Bottle) -> Result<()> {
        self.save_config(bottle)
    }

    /// Create a bottle directory and a `bottle.yml` understood by the Python
    /// implementation.  Component downloads are deliberately not performed.
    pub fn create_bottle(&self, mut options: NewBottleOptions) -> Result<Bottle> {
        self.paths.ensure_app_dirs()?;
        let name = validate_bottle_name(&options.name)?;
        options.name = name.clone();
        let mut path_name = sanitize_universal_filename(&name.replace(' ', "-"));
        let uses_default_path = options
            .path
            .as_ref()
            .map(|path| same_path(path, &self.paths.bottles))
            .unwrap_or(true);
        let mut prefix = if uses_default_path {
            self.paths.bottles.join(&path_name)
        } else {
            options.path.clone().unwrap().join(&path_name)
        };
        if prefix.exists() {
            let suffix = (uuid::Uuid::new_v4().as_u128() % 101 + 100) as u32;
            let suffixed_name = format!("{name}__{suffix}");
            path_name = format!("{path_name}__{suffix}");
            prefix = prefix.with_file_name(&path_name);
            options.name = suffixed_name;
        }
        fs::create_dir(&prefix).with_path(&prefix)?;
        let drive_c = prefix.join("drive_c");
        fs::create_dir_all(&drive_c).with_path(&drive_c)?;
        for cache in ["dxvk_shader", "gl_shader", "mesa_shader", "vkd3d_shader"] {
            fs::create_dir_all(prefix.join("cache").join(cache))
                .with_path(prefix.join("cache").join(cache))?;
        }

        let mut placeholder = None;
        if !uses_default_path {
            let placeholder_dir = self.paths.bottles.join(&path_name);
            fs::create_dir_all(&placeholder_dir).with_path(&placeholder_dir)?;
            let placeholder_file = placeholder_dir.join("placeholder.yml");
            let mut mapping = serde_yaml::Mapping::new();
            mapping.insert(
                Value::String("Path".to_owned()),
                Value::String(prefix.to_string_lossy().into_owned()),
            );
            fs::write(&placeholder_file, serde_yaml::to_string(&mapping)?)
                .with_path(&placeholder_file)?;
            placeholder = Some(placeholder_file);
        }

        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
        let mut config = BottleConfig::default();
        config.name = options.name.clone();
        config.arch = options.arch.clone();
        config.runner = options.runner.unwrap_or_else(|| self.latest_runner());
        config.d7vk = options.d7vk.unwrap_or_else(|| self.first_component(ComponentKind::D7Vk));
        config.dxvk = options.dxvk.unwrap_or_else(|| self.first_component(ComponentKind::Dxvk));
        config.vkd3d = options
            .vkd3d
            .unwrap_or_else(|| self.first_component(ComponentKind::Vkd3d));
        config.nvapi = options
            .nvapi
            .unwrap_or_else(|| self.first_component(ComponentKind::Nvapi));
        config.latency_flex = options
            .latencyflex
            .unwrap_or_else(|| self.first_component(ComponentKind::LatencyFlex));
        config.path = if uses_default_path {
            path_name
        } else {
            prefix.to_string_lossy().into_owned()
        };
        config.custom_path = CustomPath::Bool(!uses_default_path);
        config.environment = capitalize(&options.environment);
        config.creation_date = now.clone();
        config.update_date = now;
        config.parameters.sandbox = options.sandbox;
        config.versioning = options.versioning;
        config.limit_system_environment = true;
        config.inherited_environment_variables = DEFAULT_INHERITED_ENVIRONMENT
            .iter()
            .map(|value| (*value).to_owned())
            .collect();

        let template = if let Some(path) = options.custom_environment.as_deref() {
            let content = fs::read_to_string(path).with_path(path)?;
            Some(serde_yaml::from_str::<EnvironmentTemplate>(&content)?)
        } else {
            builtin_environment_template(&options.environment)
        };
        if let Some(template) = template {
            config.apply_environment_template(template)?;
        }

        let config_path = prefix.join("bottle.yml");
        config.save(&config_path)?;
        Ok(Bottle {
            directory_name: sanitize_universal_filename(&config.name.replace(' ', "-")),
            config_path,
            prefix,
            placeholder,
            config,
        })
    }

    pub fn rename_bottle(&self, name: &str, new_name: &str) -> Result<Bottle> {
        let clean = validate_bottle_name(new_name)?;
        let mut bottle = self.get_bottle(name)?;
        bottle.config.name = clean;
        bottle.config.update_date = Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
        self.save_config(&bottle)?;
        Ok(bottle)
    }

    pub fn duplicate_bottle(&self, name: &str, new_name: &str) -> Result<Bottle> {
        let clean = validate_bottle_name(new_name)?;
        let source = self.get_bottle(name)?;
        let base_name = sanitize_universal_filename(&clean.replace(' ', "-"));
        let mut duplicate_name = clean.clone();
        let mut destination_name = base_name.clone();
        let mut destination = self.paths.bottles.join(&destination_name);
        let mut suffix = 1;
        while destination.exists() {
            duplicate_name = format!("{clean}__{suffix}");
            destination_name = format!("{base_name}__{suffix}");
            destination = self.paths.bottles.join(&destination_name);
            suffix += 1;
        }
        fs::create_dir(&destination).with_path(&destination)?;
        let result = (|| -> Result<()> {
            for item in [
                "drive_c",
                "system.reg",
                "user.reg",
                "userdef.reg",
                "bottle.yml",
            ] {
                let from = source.prefix.join(item);
                let to = destination.join(item);
                if from.is_dir() {
                    copy_directory(&from, &to)?;
                } else if from.is_file() {
                    fs::copy(&from, &to).with_path(&from)?;
                }
            }
            let config_path = destination.join("bottle.yml");
            let mut config = BottleConfig::load(&config_path)?;
            config.name = duplicate_name.clone();
            config.path = destination.to_string_lossy().into_owned();
            config.custom_path = CustomPath::Text(destination.to_string_lossy().into_owned());
            config.creation_date = Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
            config.update_date = config.creation_date.clone();
            config.save(&config_path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&destination);
            return Err(result.unwrap_err());
        }
        self.get_bottle(&duplicate_name)
    }

    pub fn delete_bottle(&self, name: &str) -> Result<()> {
        let bottle = self.get_bottle(name)?;
        fs::remove_dir_all(&bottle.prefix).with_path(&bottle.prefix)?;
        if let Some(placeholder) = bottle.placeholder {
            if let Some(parent) = placeholder.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
        Ok(())
    }

    pub fn repair_bottle(&self, name: &str) -> Result<()> {
        let bottle = self.get_bottle(name)?;
        fs::create_dir_all(bottle.prefix.join("drive_c")).with_path(bottle.prefix.join("drive_c"))?;
        let launcher = WineLauncher::new(self.paths.clone(), bottle.config.clone(), bottle.prefix);
        let plan = launcher.tool_plan(WineTool::Wineboot, &["-u".to_owned()])?;
        launcher.spawn(&plan)?;
        Ok(())
    }

    pub fn add_program(
        &self,
        bottle_name: &str,
        name: &str,
        path: &str,
        launch_options: Option<&str>,
        no_d7vk: bool,
        no_dxvk: bool,
        no_vkd3d: bool,
        no_dxvk_nvapi: bool,
    ) -> Result<String> {
        let mut bottle = self.get_bottle(bottle_name)?;
        let (executable, folder) = if is_unix_path(path) {
            let unix_path = PathBuf::from(path.trim_start_matches("file://"));
            if !unix_path.exists() {
                return Err(BottlesError::InvalidInput(format!(
                    "Path doesn't exist or is unreachable: {path}"
                )));
            }
            let executable = unix_path
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            let folder = unix_path
                .parent()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            (executable, folder)
        } else if is_windows_path(path) {
            let normalized = path.replace('\\', "/");
            let executable = normalized
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_owned();
            let parent = normalized.rsplit_once('/').map(|(left, _)| left).unwrap_or("");
            let mut relative = parent.replace("C:/", "/drive_c/");
            if relative.starts_with('/') {
                relative.remove(0);
            }
            let folder = bottle.prefix.join(relative).to_string_lossy().into_owned();
            (executable, folder)
        } else {
            return Err(BottlesError::InvalidInput(format!(
                "Unsupported path type: {path}"
            )));
        };
        let id = uuid::Uuid::new_v4().to_string();
        let program = ExternalProgram {
            id: Some(id.clone()),
            name: Some(name.to_owned()),
            executable: Some(executable),
            path: Some(path.to_owned()),
            folder: Some(folder),
            icon: Some(String::new()),
            arguments: Some(launch_options.unwrap_or_default().to_owned()),
            dxvk: Some(if no_dxvk { false } else { bottle.config.parameters.dxvk }),
            vkd3d: Some(if no_vkd3d {
                false
            } else {
                bottle.config.parameters.vkd3d
            }),
            dxvk_nvapi: Some(if no_dxvk_nvapi {
                false
            } else {
                bottle.config.parameters.dxvk_nvapi
            }),
            d7vk: no_d7vk.then_some(false),
            ..ExternalProgram::default()
        };
        bottle
            .config
            .external_programs
            .insert(id.clone(), program);
        self.save_config(&bottle)?;
        Ok(id)
    }

    pub fn autostart_programs(&self) -> Result<Vec<(Bottle, ExternalProgram)>> {
        let mut result = Vec::new();
        for bottle in self.list_bottles()? {
            for program in bottle.config.external_programs.values() {
                if program.autostart == Some(true) && program.removed != Some(true) {
                    result.push((bottle.clone(), program.clone()));
                }
            }
        }
        Ok(result)
    }

    pub fn components(&self) -> BTreeMap<String, Vec<String>> {
        let mut result = BTreeMap::new();
        for kind in [
            ComponentKind::Runners,
            ComponentKind::D7Vk,
            ComponentKind::Dxvk,
            ComponentKind::Vkd3d,
            ComponentKind::Nvapi,
            ComponentKind::LatencyFlex,
        ] {
            result.insert(kind.key().to_owned(), list_directories(&self.paths.component_directory(kind)));
        }
        result
    }

    pub fn latest_runner(&self) -> String {
        let mut runners = list_directories(&self.paths.runners);
        runners.sort_by(|left, right| natural_version_cmp(left, right));
        runners
            .iter()
            .rev()
            .find(|runner| runner.to_ascii_lowercase().starts_with("soda"))
            .cloned()
            .or_else(|| runners.last().cloned())
            .unwrap_or_else(|| {
                if crate::paths::which("wine").is_some() {
                    "sys-wine".to_owned()
                } else {
                    String::new()
                }
            })
    }

    pub fn first_component(&self, kind: ComponentKind) -> String {
        list_directories(&self.paths.component_directory(kind))
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    /// Mirror `ImportManager.search_wineprefixes`.
    pub fn search_wineprefixes(&self, selected: &[PathBuf]) -> Vec<WinePrefixCandidate> {
        let legacy = LegacyPaths::detect();
        let mut candidates: Vec<(PathBuf, &str)> = Vec::new();
        if legacy.wine.is_dir() {
            candidates.push((legacy.wine, "Legacy Wine"));
        }
        for (root, manager) in [
            (legacy.lutris, "Lutris"),
            (legacy.playonlinux, "PlayOnLinux"),
            (legacy.bottles_v1, "Bottles v1"),
        ] {
            for path in list_subdirectories(&root) {
                candidates.push((path, manager));
            }
        }
        for selected_path in selected {
            let normalized = normalize_path(selected_path);
            if normalized.join("drive_c").is_dir() {
                candidates.push((normalized, "Manual"));
            } else {
                candidates.extend(
                    list_subdirectories(&normalized)
                        .into_iter()
                        .map(|path| (path, "Manual")),
                );
            }
        }
        let mut seen = std::collections::HashSet::new();
        candidates
            .into_iter()
            .filter_map(|(path, manager)| {
                if !path.join("drive_c").is_dir() || !seen.insert(path.clone()) {
                    return None;
                }
                Some(WinePrefixCandidate {
                    name: path
                        .file_name()
                        .map(|value| value.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Imported".to_owned()),
                    manager: manager.to_owned(),
                    lock: path.join("bottle.lock").is_file(),
                    path,
                })
            })
            .collect()
    }

    /// Mirror `ImportManager.import_wineprefix`.
    pub fn import_wineprefix(&self, candidate: &WinePrefixCandidate) -> Result<Bottle> {
        let path_name = format!("Imported_{}", candidate.name);
        let destination = self.paths.bottles.join(&path_name);
        fs::create_dir(&destination).with_path(&destination)?;
        let status = Command::new("cp")
            .arg("-a")
            .arg(candidate.path.join("."))
            .arg(&destination)
            .status();
        if !matches!(status, Ok(status) if status.success()) {
            let _ = fs::remove_dir_all(&destination);
            return Err(BottlesError::CommandFailed {
                command: "cp -a".to_owned(),
                message: "Could not copy the source Wine prefix".to_owned(),
            });
        }
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
        let mut config = BottleConfig::default();
        config.name = candidate.name.clone();
        config.runner = self.latest_runner();
        config.path = path_name.clone();
        config.environment = "Custom".to_owned();
        config.creation_date = now.clone();
        config.update_date = now;
        let config_path = destination.join("bottle.yml");
        if let Err(error) = config.save(&config_path) {
            let _ = fs::remove_dir_all(&destination);
            return Err(error);
        }
        let _ = fs::write(candidate.path.join("bottle.lock"), "");
        self.get_bottle(&candidate.name)
    }

    /// A simple full-prefix backup.  The Python FVS versioning system remains
    /// the authoritative implementation for incremental commits.
    pub fn create_backup(&self, bottle_name: &str, destination: &Path) -> Result<PathBuf> {
        let bottle = self.get_bottle(bottle_name)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_path(parent)?;
        }
        let status = Command::new("tar")
            .arg("-czf")
            .arg(destination)
            .arg("-C")
            .arg(&bottle.prefix)
            .arg(".")
            .status()
            .map_err(|source| BottlesError::Io {
                path: PathBuf::from("tar"),
                source,
            })?;
        if !status.success() {
            return Err(BottlesError::CommandFailed {
                command: "tar -czf".to_owned(),
                message: "Could not create bottle backup".to_owned(),
            });
        }
        Ok(destination.to_path_buf())
    }
}

pub fn bottle_prefix_path(paths: &Paths, config: &BottleConfig) -> PathBuf {
    if config.environment == "Steam" {
        return paths.steam.join(&config.compat_data);
    }
    if config.custom_path.is_truthy() {
        return PathBuf::from(&config.path);
    }
    paths.bottles.join(&config.path)
}

pub fn builtin_environment_template(name: &str) -> Option<EnvironmentTemplate> {
    let mut parameters = BTreeMap::new();
    let installed_dependencies = match name.to_ascii_lowercase().as_str() {
        "gaming" => {
            parameters.insert("dxvk".to_owned(), Value::Bool(true));
            parameters.insert("vkd3d".to_owned(), Value::Bool(true));
            parameters.insert("sync".to_owned(), Value::String("ntsync".to_owned()));
            parameters.insert("fsr".to_owned(), Value::Bool(false));
            parameters.insert("discrete_gpu".to_owned(), Value::Bool(true));
            parameters.insert("pulseaudio_latency".to_owned(), Value::Bool(false));
            vec![
                "d3dx9",
                "msls31",
                "arial32",
                "times32",
                "courie32",
                "d3dcompiler_43",
                "d3dcompiler_47",
                "mono",
                "gecko",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        }
        "application" => {
            parameters.insert("dxvk".to_owned(), Value::Bool(true));
            parameters.insert("vkd3d".to_owned(), Value::Bool(true));
            parameters.insert("pulseaudio_latency".to_owned(), Value::Bool(false));
            vec!["arial32", "times32", "courie32", "mono", "gecko"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        }
        _ => return None,
    };
    Some(EnvironmentTemplate {
        runner: Some("wine".to_owned()),
        parameters,
        installed_dependencies,
    })
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    result
}

fn list_directories(path: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut result = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn list_subdirectories(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut result = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_path(destination)?;
    for entry in fs::read_dir(source).with_path(source)?.filter_map(|entry| entry.ok()) {
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&from).with_path(&from)?;
        if metadata.is_symlink() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let target = fs::read_link(&from).with_path(&from)?;
                symlink(target, &to).with_path(&to)?;
            }
            #[cfg(not(unix))]
            {
                if from.is_dir() {
                    copy_directory(&from, &to)?;
                } else {
                    fs::copy(&from, &to).with_path(&from)?;
                }
            }
        } else if metadata.is_dir() {
            copy_directory(&from, &to)?;
        } else {
            fs::copy(&from, &to).with_path(&from)?;
        }
    }
    Ok(())
}

fn natural_version_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = left.split(|c: char| !c.is_ascii_alphanumeric()).collect::<Vec<_>>();
    let right_parts = right.split(|c: char| !c.is_ascii_alphanumeric()).collect::<Vec<_>>();
    for (a, b) in left_parts.iter().zip(right_parts.iter()) {
        match (a.parse::<u64>(), b.parse::<u64>()) {
            (Ok(a), Ok(b)) => {
                let ordering = a.cmp(&b);
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }
            _ => {
                let ordering = a.cmp(b);
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }
        }
    }
    left_parts.len().cmp(&right_parts.len()).then_with(|| left.cmp(right))
}

fn is_unix_path(path: &str) -> bool {
    let clean = path.trim_start_matches("file://");
    clean.starts_with('/') || clean.starts_with("./") || clean.starts_with("../")
}

fn is_windows_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\')
        || path.starts_with("\\\\")
        || path.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(name: &str) -> Paths {
        let mut paths = Paths::detect();
        paths.base = std::env::temp_dir().join(format!("bottles-rs-{name}-{}", uuid::Uuid::new_v4()));
        paths.bottles = paths.base.join("bottles");
        paths.runners = paths.base.join("runners");
        paths.temp = paths.base.join("temp");
        paths
    }

    #[test]
    fn create_and_list_a_bottle_with_python_compatible_config() {
        let paths = temp_paths("create");
        let manager = BottleManager::new(paths.clone());
        let bottle = manager
            .create_bottle(NewBottleOptions {
                name: "My Game".to_owned(),
                environment: "gaming".to_owned(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(bottle.config.name, "My Game");
        assert_eq!(bottle.config.environment, "Gaming");
        assert!(bottle.config.parameters.dxvk);
        assert_eq!(bottle.config.parameters.sync, "ntsync");
        assert!(bottle.config_path.is_file());
        let listed = manager.list_bottles().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].config.name, "My Game");
        fs::remove_dir_all(paths.base).unwrap();
    }

    #[test]
    fn rename_duplicate_and_delete_preserve_config_names() {
        let paths = temp_paths("lifecycle");
        let manager = BottleManager::new(paths.clone());
        manager
            .create_bottle(NewBottleOptions {
                name: "Original".to_owned(),
                ..Default::default()
            })
            .unwrap();
        manager.rename_bottle("Original", "Renamed").unwrap();
        let duplicate = manager.duplicate_bottle("Renamed", "Copy").unwrap();
        assert_eq!(duplicate.config.name, "Copy");
        assert_eq!(manager.list_bottles().unwrap().len(), 2);
        manager.delete_bottle("Copy").unwrap();
        assert_eq!(manager.list_bottles().unwrap().len(), 1);
        fs::remove_dir_all(paths.base).unwrap();
    }

    #[test]
    fn bottle_prefix_path_matches_python_rules() {
        let paths = temp_paths("prefix");
        let mut config = BottleConfig::default();
        config.path = "example".to_owned();
        assert_eq!(bottle_prefix_path(&paths, &config), paths.bottles.join("example"));
        config.custom_path = CustomPath::Bool(true);
        config.path = "/mnt/games/example".to_owned();
        assert_eq!(bottle_prefix_path(&paths, &config), PathBuf::from("/mnt/games/example"));
        config.environment = "Steam".to_owned();
        config.compat_data = "123".to_owned();
        assert_eq!(bottle_prefix_path(&paths, &config), paths.steam.join("123"));
        fs::remove_dir_all(paths.base).ok();
    }

    #[test]
    fn custom_path_placeholder_is_followed() {
        let paths = temp_paths("placeholder");
        let manager = BottleManager::new(paths.clone());
        let external = paths.base.join("external");
        fs::create_dir_all(&external).unwrap();
        manager
            .create_bottle(NewBottleOptions {
                name: "External".to_owned(),
                path: Some(external.clone()),
                ..Default::default()
            })
            .unwrap();
        let bottles = manager.list_bottles().unwrap();
        assert_eq!(bottles.len(), 1);
        assert!(bottles[0].placeholder.is_some());
        assert_eq!(bottles[0].prefix, external.join("External"));
        fs::remove_dir_all(paths.base).unwrap();
    }
}

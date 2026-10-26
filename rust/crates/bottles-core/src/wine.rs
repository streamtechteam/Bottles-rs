use crate::config::{BottleConfig, ExternalProgram};
use crate::error::{BottlesError, Result};
use crate::paths::{which, Paths, EXECUTABLE_EXTENSIONS};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

pub const PROGRAM_ENVIRONMENT_DENYLIST: &[&str] = &["WINEARCH", "WINEPREFIX"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WineTool {
    Cmd,
    WineCfg,
    Uninstaller,
    Regedit,
    Taskmgr,
    Control,
    Explorer,
    Reg,
    Wineboot,
    Wineserver,
    Winepath,
    Start,
    MsiExec,
    Notepad,
    Regsvr32,
    Rundll32,
    WinHelp,
    Xcopy,
    OleView,
    IcInfo,
    Hh,
    Progman,
    Winefile,
    Winedbg,
}

impl WineTool {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "cmd" => Self::Cmd,
            "winecfg" => Self::WineCfg,
            "uninstaller" => Self::Uninstaller,
            "regedit" => Self::Regedit,
            "taskmgr" => Self::Taskmgr,
            "control" => Self::Control,
            "explorer" => Self::Explorer,
            "reg" => Self::Reg,
            "wineboot" => Self::Wineboot,
            "wineserver" => Self::Wineserver,
            "winepath" => Self::Winepath,
            "start" => Self::Start,
            "msiexec" => Self::MsiExec,
            "notepad" => Self::Notepad,
            "regsvr32" => Self::Regsvr32,
            "rundll32" => Self::Rundll32,
            "winhelp" => Self::WinHelp,
            "hh" => Self::Hh,
            "xcopy" => Self::Xcopy,
            "oleview" => Self::OleView,
            "icinfo" => Self::IcInfo,
            "progman" => Self::Progman,
            "winefile" => Self::Winefile,
            "winedbg" => Self::Winedbg,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Cmd => "cmd",
            Self::WineCfg => "winecfg",
            Self::Uninstaller => "uninstaller",
            Self::Regedit => "regedit",
            Self::Taskmgr => "taskmgr",
            Self::Control => "control",
            Self::Explorer => "explorer",
            Self::Reg => "reg",
            Self::Wineboot => "wineboot",
            Self::Wineserver => "wineserver",
            Self::Winepath => "winepath",
            Self::Start => "start",
            Self::MsiExec => "msiexec",
            Self::Notepad => "notepad",
            Self::Regsvr32 => "regsvr32",
            Self::Rundll32 => "rundll32",
            Self::WinHelp => "winhelp",
            Self::Hh => "hh",
            Self::Xcopy => "xcopy",
            Self::OleView => "oleview",
            Self::IcInfo => "icinfo",
            Self::Progman => "progman",
            Self::Winefile => "winefile",
            Self::Winedbg => "winedbg",
        }
    }

    pub fn cli_names() -> &'static [&'static str] {
        &[
            "cmd",
            "winecfg",
            "uninstaller",
            "regedit",
            "taskmgr",
            "control",
            "explorer",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub runner: PathBuf,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub working_directory: Option<PathBuf>,
    pub clear_environment: bool,
    pub executable: PathBuf,
    pub prefix: PathBuf,
}

/// Build Wine command lines and the environment represented by `WineCommand`.
/// GPU/runtime/sandbox refinements from the Python implementation are tracked
/// as partial in the generated parity inventory.
#[derive(Debug, Clone)]
pub struct WineLauncher {
    pub paths: Paths,
    pub config: BottleConfig,
    pub prefix: PathBuf,
}

impl WineLauncher {
    pub fn new(paths: Paths, config: BottleConfig, prefix: PathBuf) -> Self {
        Self {
            paths,
            config,
            prefix,
        }
    }

    pub fn runner(&self) -> Result<PathBuf> {
        if !self.config.runner_path.is_empty() {
            let path = PathBuf::from(&self.config.runner_path);
            if path.exists() {
                return Ok(path);
            }
        }
        if self.config.runner.starts_with("sys-") {
            let wine64 = which("wine64");
            let wine = which("wine");
            let resolved = if self.config.arch == "win64" {
                wine64.or(wine)
            } else {
                wine.or(wine64)
            };
            return resolved.ok_or_else(|| BottlesError::CommandFailed {
                command: "wine".to_owned(),
                message: "system Wine runner was not found".to_owned(),
            });
        }
        let root = self.paths.runners.join(&self.config.runner);
        let candidates = [
            root.join("bin/wine64"),
            root.join("bin/wine"),
            root.join("dist/bin/wine64"),
            root.join("dist/bin/wine"),
            root.join("files/bin/wine64"),
            root.join("files/bin/wine"),
            root.join("proton"),
            root.clone(),
        ];
        candidates
            .iter()
            .find(|candidate| candidate.is_file())
            .cloned()
            .ok_or_else(|| BottlesError::CommandFailed {
                command: self.config.runner.clone(),
                message: format!("runner was not found under {}", root.display()),
            })
    }

    /// Build a clean environment for standalone scripts or an inherited
    /// environment for a regular launch.
    pub fn environment(
        &self,
        extra: &BTreeMap<String, String>,
        clean: bool,
    ) -> BTreeMap<String, String> {
        let mut environment = BTreeMap::new();
        if clean {
            // GSettings and XDG values needed by Wine are supplied explicitly.
        } else if self.config.limit_system_environment {
            let allowed: HashSet<&str> = self
                .config
                .inherited_environment_variables
                .iter()
                .map(String::as_str)
                .collect();
            for (key, value) in std::env::vars() {
                if allowed.contains(key.as_str()) {
                    environment.insert(key, value);
                }
            }
        } else {
            for (key, value) in std::env::vars() {
                environment.insert(key, value);
            }
        }

        environment.insert("BOTTLE".to_owned(), self.config.path.clone());
        environment.insert(
            "WINEPREFIX".to_owned(),
            self.prefix.to_string_lossy().into_owned(),
        );
        environment.insert("WINEARCH".to_owned(), self.config.arch.clone());
        environment.insert(
            "WINEDEBUG".to_owned(),
            if self.config.parameters.fixme_logs {
                "+fixme-all"
            } else {
                "fixme-all"
            }
            .to_owned(),
        );
        if self.config.parameters.latencyflex {
            environment.remove("DISABLE_LFX");
            let layer = self
                .paths
                .component_path(
                    crate::paths::ComponentKind::LatencyFlex,
                    &self.config.latency_flex,
                )
                .join("layer/usr/share/vulkan/implicit_layer.d");
            append_env_value(
                &mut environment,
                "VK_ADD_LAYER_PATH",
                &layer.to_string_lossy(),
                ":",
            );
            environment.insert("LFX".to_owned(), "1".to_owned());
        } else {
            environment.insert("DISABLE_LFX".to_owned(), "1".to_owned());
        }

        for (key, value) in &self.config.environment_variables {
            if let Some(value) = value.as_str() {
                environment.insert(key.clone(), value.to_owned());
            }
        }

        if self.config.language != "sys" {
            let language = if self.config.language.contains('.') {
                self.config.language.clone()
            } else {
                format!("{}.UTF-8", self.config.language)
            };
            environment.insert("LC_ALL".to_owned(), language);
        }

        let params = &self.config.parameters;
        if params.dxvk {
            environment.insert("WINE_LARGE_ADDRESS_AWARE".to_owned(), "1".to_owned());
            environment.insert(
                "DXVK_SHADER_CACHE_PATH".to_owned(),
                self.prefix
                    .join("cache/dxvk_shader")
                    .to_string_lossy()
                    .into_owned(),
            );
            environment.insert("STAGING_SHARED_MEMORY".to_owned(), "1".to_owned());
            environment.insert("__GL_SHADER_DISK_CACHE".to_owned(), "1".to_owned());
            environment.insert(
                "__GL_SHADER_DISK_CACHE_PATH".to_owned(),
                self.prefix
                    .join("cache/gl_shader")
                    .to_string_lossy()
                    .into_owned(),
            );
            environment.insert(
                "MESA_SHADER_CACHE_DIR".to_owned(),
                self.prefix
                    .join("cache/mesa_shader")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        if params.vkd3d {
            environment.insert(
                "VKD3D_SHADER_CACHE_PATH".to_owned(),
                self.prefix
                    .join("cache/vkd3d_shader")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        if params.dxvk_nvapi {
            environment.insert("DXVK_NVAPIHACK".to_owned(), "0".to_owned());
            environment.insert("DXVK_ENABLE_NVAPI".to_owned(), "1".to_owned());
        }
        if params.mangohud {
            environment.insert("MANGOHUD".to_owned(), "1".to_owned());
            environment.insert("MANGOHUD_DLSYM".to_owned(), "1".to_owned());
            if !params.mangohud_display_on_game_start {
                environment.insert(
                    "MANGOHUD_CONFIG".to_owned(),
                    "read_cfg,no_display".to_owned(),
                );
            }
        }
        if params.vkbasalt {
            environment.insert("ENABLE_VKBASALT".to_owned(), "1".to_owned());
            let config = self.prefix.join("vkBasalt.conf");
            if config.is_file() {
                environment.insert(
                    "VKBASALT_CONFIG_FILE".to_owned(),
                    config.to_string_lossy().into_owned(),
                );
            }
        }
        if params.obsvkc {
            environment.insert("OBS_VKCAPTURE".to_owned(), "1".to_owned());
        }
        if params.gamescope {
            environment.insert("GAMESCOPE".to_owned(), "1".to_owned());
        }
        if params.discrete_gpu {
            environment.insert("DRI_PRIME".to_owned(), "1".to_owned());
        }
        if params.pulseaudio_latency {
            environment.insert("PULSE_LATENCY_MSEC".to_owned(), "60".to_owned());
        }
        if params.use_eac_runtime {
            environment.insert("PROTON_EAC_RUNTIME".to_owned(), "1".to_owned());
        }
        if params.use_be_runtime {
            environment.insert("PROTON_BATTLEYE_RUNTIME".to_owned(), "1".to_owned());
        }
        match params.sync.as_str() {
            "esync" => {
                environment.insert("WINEESYNC".to_owned(), "1".to_owned());
            }
            "fsync" => {
                environment.insert("WINEFSYNC".to_owned(), "1".to_owned());
            }
            "wine" | "ntsync" => {
                environment.insert("WINENTSYNC".to_owned(), "1".to_owned());
            }
            _ => {}
        }
        if params.frame_rate_limit > 0 {
            let limit = params.frame_rate_limit;
            let frame = format!("dxgi.maxFrameRate = {limit}; d3d9.maxFrameRate = {limit}");
            let config = environment
                .get("DXVK_CONFIG")
                .map(|existing| format!("{}; {frame}", existing.trim_end_matches([';', ' '])))
                .unwrap_or(frame);
            environment.insert("DXVK_CONFIG".to_owned(), config);
            environment.insert("VKD3D_FRAME_RATE".to_owned(), limit.to_string());
        }
        let hidraw = normalized_hidraw_ids(&params.hidraw_devices);
        if !hidraw.is_empty() {
            environment.insert("PROTON_ENABLE_HIDRAW".to_owned(), hidraw.join(","));
        }
        apply_lsfg_vk_environment(&mut environment, self);

        let mut overrides = Vec::new();
        for (key, value) in &self.config.dll_overrides {
            overrides.push(format!("{key}={value}"));
        }
        overrides.push("winemenubuilder=''".to_owned());
        append_env_value(
            &mut environment,
            "WINEDLLOVERRIDES",
            &overrides.join(";"),
            ";",
        );

        for (key, value) in extra {
            if PROGRAM_ENVIRONMENT_DENYLIST.contains(&key.as_str()) {
                continue;
            }
            if key == "WINEDLLOVERRIDES" {
                append_env_value(&mut environment, key, value, ";");
            } else {
                environment.insert(key.clone(), value.clone());
            }
        }
        environment
    }

    pub fn program_environment(&self, program: &ExternalProgram) -> BTreeMap<String, String> {
        let mut extra = program.environment.clone().unwrap_or_default();
        let mut overrides = Vec::new();
        if program.d7vk == Some(false) && self.config.parameters.d7vk {
            overrides.push("ddraw=b".to_owned());
        }
        let dxvk_enabled = program.dxvk.unwrap_or(self.config.parameters.dxvk);
        if !dxvk_enabled {
            overrides.push("d3d8,d3d9,d3d10core,d3d11,dxgi=b".to_owned());
        }
        if program.vkd3d == Some(false) && self.config.parameters.vkd3d {
            overrides.push("d3d12,d3d12core=b".to_owned());
        }
        if program.dxvk_nvapi == Some(false) && self.config.parameters.dxvk_nvapi {
            overrides.push("nvapi,nvapi64=b".to_owned());
        }
        if !overrides.is_empty() {
            let current = extra.get("WINEDLLOVERRIDES").cloned().unwrap_or_default();
            extra.insert(
                "WINEDLLOVERRIDES".to_owned(),
                if current.is_empty() {
                    overrides.join(";")
                } else {
                    format!("{current};{}", overrides.join(";"))
                },
            );
        }
        if let Some(gamescope) = program.gamescope {
            if gamescope != self.config.parameters.gamescope {
                extra.insert(
                    "GAMESCOPE".to_owned(),
                    if gamescope { "1" } else { "0" }.to_owned(),
                );
            }
        }
        extra
    }

    pub fn plan(
        &self,
        executable: &str,
        arguments: &[String],
        cwd: Option<PathBuf>,
    ) -> Result<LaunchPlan> {
        self.plan_with_environment(executable, arguments, cwd, &BTreeMap::new(), false)
    }

    pub fn plan_with_environment(
        &self,
        executable: &str,
        arguments: &[String],
        cwd: Option<PathBuf>,
        extra_environment: &BTreeMap<String, String>,
        clean: bool,
    ) -> Result<LaunchPlan> {
        let runner = self.runner()?;
        let clean_executable = executable.trim_matches('"').trim_matches('\'').to_owned();
        let executable_path = PathBuf::from(&clean_executable);
        let extension = executable_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{}", value.to_ascii_lowercase()))
            .unwrap_or_default();
        let mut command_arguments = match extension.as_str() {
            ".msi" => vec![
                "msiexec".to_owned(),
                "/i".to_owned(),
                clean_executable.clone(),
            ],
            ".bat" => vec!["cmd".to_owned(), "/c".to_owned(), clean_executable.clone()],
            ".lnk" => vec![
                "start".to_owned(),
                "/unix".to_owned(),
                clean_executable.clone(),
            ],
            _ => vec![clean_executable.clone()],
        };
        command_arguments.extend(arguments.iter().cloned());
        Ok(LaunchPlan {
            runner,
            arguments: command_arguments,
            environment: self.environment(extra_environment, clean),
            working_directory: cwd.filter(|path| !path.as_os_str().is_empty()),
            clear_environment: clean || self.config.limit_system_environment,
            executable: executable_path,
            prefix: self.prefix.clone(),
        })
    }

    pub fn program_plan(
        &self,
        program: &ExternalProgram,
        extra_args: &[String],
    ) -> Result<LaunchPlan> {
        let mut arguments = String::new();
        if program.arguments_enabled.unwrap_or(true) {
            arguments = program.arguments.clone().unwrap_or_default();
        }
        let arguments = resolve_placeholders(&arguments, &self.config, &self.prefix, program);
        let mut all_arguments = split_arguments(&arguments);
        all_arguments.extend(extra_args.iter().cloned());
        let path = resolve_placeholders(
            program.path.as_deref().unwrap_or_default(),
            &self.config,
            &self.prefix,
            program,
        );
        let folder = program
            .folder
            .as_ref()
            .map(|value| resolve_placeholders(value, &self.config, &self.prefix, program))
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        self.plan_with_environment(
            &path,
            &all_arguments,
            folder,
            &self.program_environment(program),
            false,
        )
    }

    pub fn tool_plan(&self, tool: WineTool, arguments: &[String]) -> Result<LaunchPlan> {
        Ok(LaunchPlan {
            runner: self.runner()?,
            arguments: std::iter::once(tool.name().to_owned())
                .chain(arguments.iter().cloned())
                .collect(),
            environment: self.environment(&BTreeMap::new(), false),
            working_directory: Some(self.prefix.clone()),
            clear_environment: self.config.limit_system_environment,
            executable: PathBuf::from(tool.name()),
            prefix: self.prefix.clone(),
        })
    }

    pub fn command(&self, plan: &LaunchPlan) -> Command {
        let mut command = Command::new(&plan.runner);
        command.args(&plan.arguments);
        if plan.clear_environment {
            command.env_clear();
        }
        command.envs(&plan.environment);
        if let Some(cwd) = &plan.working_directory {
            command.current_dir(cwd);
        }
        command.stdin(Stdio::null());
        command
    }

    pub fn spawn(&self, plan: &LaunchPlan) -> Result<Child> {
        let mut command = self.command(plan);
        command.spawn().map_err(|source| BottlesError::Io {
            path: plan.runner.clone(),
            source,
        })
    }

    pub fn execute(&self, plan: &LaunchPlan) -> Result<Output> {
        let mut command = self.command(plan);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.output().map_err(|source| BottlesError::Io {
            path: plan.runner.clone(),
            source,
        })
    }

    pub fn standalone_script(&self) -> Result<String> {
        let environment = self.environment(&BTreeMap::new(), true);
        let mut script = String::from("#!/usr/bin/env bash\n");
        for (key, value) in environment {
            script.push_str(&format!("export {key}='{}'\n", shell_single_quote(&value)));
        }
        let runner = self.runner()?;
        script.push_str(&format!(
            "'{}' \"$@\"\n",
            shell_single_quote(&runner.to_string_lossy())
        ));
        Ok(script)
    }
}

fn append_env_value(
    environment: &mut BTreeMap<String, String>,
    key: &str,
    value: &str,
    separator: &str,
) {
    if value.is_empty() {
        return;
    }
    match environment.get_mut(key) {
        Some(existing) if !existing.is_empty() => {
            existing.push_str(separator);
            existing.push_str(value);
        }
        _ => {
            environment.insert(key.to_owned(), value.to_owned());
        }
    }
}

fn apply_lsfg_vk_environment(environment: &mut BTreeMap<String, String>, launcher: &WineLauncher) {
    let params = &launcher.config.parameters;
    if !params.lsfg_vk {
        return;
    }
    let dll = launcher.prefix.join("lsfg-vk/Lossless.dll");
    let version = crate::paths::ToolAvailability::detect().lsfg_vk_version;
    let enabled = (version == 1 || version == 2)
        && dll.is_file()
        && (2..=4).contains(&params.lsfg_vk_multiplier)
        && (0.25..=1.0).contains(&params.lsfg_vk_flow_scale);
    if !enabled {
        environment.insert("DISABLE_LSFG".to_owned(), "1".to_owned());
        environment.insert("DISABLE_LSFGVK".to_owned(), "1".to_owned());
        return;
    }
    let multiplier = params.lsfg_vk_multiplier.to_string();
    let flow_scale = params.lsfg_vk_flow_scale.to_string();
    let performance = if params.lsfg_vk_performance_mode {
        "1"
    } else {
        "0"
    };
    if version == 2 {
        environment.insert("DISABLE_LSFG".to_owned(), "1".to_owned());
        environment.remove("DISABLE_LSFGVK");
        environment.insert("LSFGVK_ENV".to_owned(), "1".to_owned());
        environment.insert(
            "LSFGVK_DLL_PATH".to_owned(),
            dll.to_string_lossy().into_owned(),
        );
        environment.insert("LSFGVK_MULTIPLIER".to_owned(), multiplier);
        environment.insert("LSFGVK_FLOW_SCALE".to_owned(), flow_scale);
        environment.insert("LSFGVK_PERFORMANCE_MODE".to_owned(), performance.to_owned());
    } else {
        environment.remove("DISABLE_LSFG");
        environment.insert("DISABLE_LSFGVK".to_owned(), "1".to_owned());
        environment.insert("LSFG_LEGACY".to_owned(), "1".to_owned());
        environment.insert(
            "LSFG_DLL_PATH".to_owned(),
            dll.to_string_lossy().into_owned(),
        );
        environment.insert("LSFG_MULTIPLIER".to_owned(), multiplier);
        environment.insert("LSFG_FLOW_SCALE".to_owned(), flow_scale);
        environment.insert("LSFG_PERFORMANCE_MODE".to_owned(), performance.to_owned());
    }
}

fn normalized_hidraw_ids(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for value in values {
        let trimmed = value.trim();
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() != 2 || parts[0].len() != 6 || parts[1].len() != 6 {
            continue;
        }
        let parse = |part: &str| {
            let digits = part
                .strip_prefix("0x")
                .or_else(|| part.strip_prefix("0X"))?;
            if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            Some(digits.to_ascii_uppercase())
        };
        if let (Some(vendor), Some(product)) = (parse(parts[0]), parse(parts[1])) {
            let id = format!("0x{vendor}/0x{product}");
            if seen.insert(id.clone()) {
                result.push(id);
            }
        }
    }
    result
}

pub fn resolve_placeholders(
    value: &str,
    config: &BottleConfig,
    prefix: &Path,
    program: &ExternalProgram,
) -> String {
    let program_path = program.path.clone().unwrap_or_default();
    let program_dir = program
        .folder
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            Path::new(&program_path)
                .parent()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
    let program_name = Path::new(&program_path)
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let replacements = [
        (
            "%PROGRAM_NAME%",
            program.executable.clone().unwrap_or(program_name.clone()),
        ),
        ("%PROGRAM_PATH%", program_path),
        ("%PROGRAM_DIR%", program_dir),
        ("%BOTTLE_NAME%", config.name.clone()),
        ("%BOTTLE_PATH%", prefix.to_string_lossy().into_owned()),
    ];
    let mut result = value.to_owned();
    for (needle, replacement) in replacements {
        result = result.replace(needle, &replacement);
    }
    result
}

/// Split a command line into arguments, honoring simple single/double quotes
/// and backslash escapes.  This mirrors the subset of `shlex.split` that the
/// Python CLI accepts for stored program arguments.
pub fn split_arguments(input: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match quote {
            Some(active) if character == active => quote = None,
            Some(_) => current.push(character),
            None if character == '\\' => escaped = true,
            None if character == '\'' || character == '"' => quote = Some(character),
            None if character.is_whitespace() => {
                if !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                }
            }
            None => current.push(character),
        }
    }
    if !current.is_empty() {
        arguments.push(current);
    }
    arguments
}

pub fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

pub fn serialize_arguments(arguments: &[String]) -> String {
    arguments
        .iter()
        .map(|argument| {
            if argument.contains(' ') || argument.contains('\'') || argument.contains('"') {
                format!(
                    "\"{}\"",
                    argument.replace('\\', "\\\\").replace('"', "\\\"")
                )
            } else {
                argument.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_windows_executable(path: &str) -> bool {
    let lowercase = path.to_ascii_lowercase();
    EXECUTABLE_EXTENSIONS
        .iter()
        .any(|extension| lowercase.ends_with(extension))
}

pub fn os_string_to_string(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}

pub fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_arguments_handles_quotes_and_escapes() {
        assert_eq!(
            split_arguments("-windowed -name \"My Game\" --foo='bar baz'"),
            vec![
                "-windowed".to_owned(),
                "-name".to_owned(),
                "My Game".to_owned(),
                "--foo=bar baz".to_owned()
            ]
        );
    }

    #[test]
    fn serialized_arguments_round_trip() {
        let original = vec!["-a".to_owned(), "two words".to_owned()];
        let serialized = serialize_arguments(&original);
        assert_eq!(split_arguments(&serialized), original);
    }

    #[test]
    fn hidraw_ids_are_normalized_and_deduplicated() {
        assert_eq!(
            normalized_hidraw_ids(&[
                "0x1234/0xabcd".to_owned(),
                "0x1234/0xabcd".to_owned(),
                "bad".to_owned()
            ]),
            vec!["0x1234/0xABCD".to_owned()]
        );
    }

    #[test]
    fn plan_dispatches_windows_file_types() {
        let paths = Paths::detect();
        let mut config = BottleConfig::default();
        config.runner = "sys-wine".to_owned();
        let launcher = WineLauncher::new(paths, config, PathBuf::from("/tmp/prefix"));
        if launcher.runner().is_ok() {
            let msi = launcher.plan("setup.msi", &[], None).unwrap();
            assert_eq!(msi.arguments[0], "msiexec");
            assert_eq!(msi.arguments[1], "/i");
        }
    }
}

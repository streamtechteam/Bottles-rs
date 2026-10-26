use crate::error::{io_error, IoResultExt, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const APP_ID: &str = "com.usebottles.bottles";
pub const APP_NAME: &str = "Bottles";
pub const APP_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR")
);
pub const DOC_URL: &str = "https://docs.usebottles.com";
pub const EXECUTABLE_EXTENSIONS: &[&str] = &[".exe", ".msi", ".bat", ".lnk"];

/// Paths mirrored from `bottles/backend/globals.py`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Paths {
    pub xdg_data_home: PathBuf,
    pub icons_user: PathBuf,
    pub base: PathBuf,
    pub applications: PathBuf,
    pub temp: PathBuf,
    pub runtimes: PathBuf,
    pub winebridge: PathBuf,
    pub runners: PathBuf,
    pub bottles: PathBuf,
    pub steam: PathBuf,
    pub d7vk: PathBuf,
    pub dxvk: PathBuf,
    pub vkd3d: PathBuf,
    pub nvapi: PathBuf,
    pub latencyflex: PathBuf,
    pub templates: PathBuf,
    pub library: PathBuf,
    pub process_metrics: PathBuf,
    pub data: PathBuf,
    pub journal: PathBuf,
    pub umu: PathBuf,
}

/// External Wine prefix locations mirrored from `TrdyPaths`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LegacyPaths {
    pub wine: PathBuf,
    pub lutris: PathBuf,
    pub playonlinux: PathBuf,
    pub bottles_v1: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ToolAvailability {
    pub gamemode: bool,
    pub gamescope: bool,
    pub hdr_wsi: bool,
    pub vkbasalt: bool,
    pub lsfg_vk: bool,
    pub lsfg_vk_version: u8,
    pub mangohud: bool,
    pub obs_vkcapture: bool,
    pub vmtouch: bool,
    pub wine: bool,
    pub vulkaninfo: bool,
    pub cabextract: bool,
    pub p7zip: bool,
    pub icoextract: bool,
    pub base_version: String,
}

impl Paths {
    pub fn detect() -> Self {
        let xdg_data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().join(".local/share"));
        let base = wine_compatible_base(&xdg_data_home.join("bottles"));
        Self {
            icons_user: xdg_data_home.join("icons"),
            applications: xdg_data_home.join("applications"),
            temp: base.join("temp"),
            runtimes: base.join("runtimes"),
            winebridge: base.join("winebridge"),
            runners: base.join("runners"),
            bottles: base.join("bottles"),
            steam: base.join("steam"),
            d7vk: base.join("d7vk"),
            dxvk: base.join("dxvk"),
            vkd3d: base.join("vkd3d"),
            nvapi: base.join("nvapi"),
            latencyflex: base.join("latencyflex"),
            templates: base.join("templates"),
            library: base.join("library.yml"),
            process_metrics: base.join("process_metrics.sqlite"),
            data: base.join("data.yml"),
            journal: base.join("journal.yml"),
            umu: base.join("umu"),
            xdg_data_home,
            base,
        }
    }

    /// Create the directories that `Manager.check_app_dirs()` creates.
    pub fn ensure_app_dirs(&self) -> Result<()> {
        for path in self.app_directories() {
            fs::create_dir_all(&path).with_path(&path)?;
        }
        Ok(())
    }

    pub fn app_directories(&self) -> Vec<PathBuf> {
        vec![
            self.base.clone(),
            self.temp.clone(),
            self.runtimes.clone(),
            self.winebridge.clone(),
            self.runners.clone(),
            self.bottles.clone(),
            self.steam.clone(),
            self.d7vk.clone(),
            self.dxvk.clone(),
            self.vkd3d.clone(),
            self.nvapi.clone(),
            self.latencyflex.clone(),
            self.templates.clone(),
            self.icons_user.clone(),
            self.applications.clone(),
            self.umu.join("games"),
            self.umu.join("prefixes"),
            self.umu.join("covers"),
        ]
    }

    pub fn component_directory(&self, component: ComponentKind) -> PathBuf {
        match component {
            ComponentKind::Runners => self.runners.clone(),
            ComponentKind::Runtimes => self.runtimes.clone(),
            ComponentKind::WineBridge => self.winebridge.clone(),
            ComponentKind::D7Vk => self.d7vk.clone(),
            ComponentKind::Dxvk => self.dxvk.clone(),
            ComponentKind::Vkd3d => self.vkd3d.clone(),
            ComponentKind::Nvapi => self.nvapi.clone(),
            ComponentKind::LatencyFlex => self.latencyflex.clone(),
        }
    }

    pub fn component_path(&self, component: ComponentKind, name: &str) -> PathBuf {
        self.component_directory(component).join(name)
    }
}

impl LegacyPaths {
    pub fn detect() -> Self {
        let home = home_dir();
        Self {
            wine: home.join(".wine"),
            lutris: home.join("Games"),
            playonlinux: home.join(".PlayOnLinux/wineprefix"),
            bottles_v1: home.join(".Bottles"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ComponentKind {
    Runners,
    Runtimes,
    WineBridge,
    D7Vk,
    Dxvk,
    Vkd3d,
    Nvapi,
    LatencyFlex,
}

impl ComponentKind {
    pub const ALL: &'static [Self] = &[
        Self::Runners,
        Self::Runtimes,
        Self::WineBridge,
        Self::D7Vk,
        Self::Dxvk,
        Self::Vkd3d,
        Self::Nvapi,
        Self::LatencyFlex,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Runners => "runners",
            Self::Runtimes => "runtimes",
            Self::WineBridge => "winebridge",
            Self::D7Vk => "d7vk",
            Self::Dxvk => "dxvk",
            Self::Vkd3d => "vkd3d",
            Self::Nvapi => "nvapi",
            Self::LatencyFlex => "latencyflex",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.key() == value.to_ascii_lowercase())
    }
}

pub fn is_cpak() -> bool {
    env::var_os("CPAK_CONTAINER_ID").is_some()
}

pub fn is_official_package() -> bool {
    env::var_os("FLATPAK_ID").is_some() || is_cpak()
}

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn locale_encodings() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([("ja_JP", "cp932"), ("zh_CN", "gbk")])
}

/// Search PATH for an executable, mirroring `shutil.which` for our purposes.
pub fn which(command: &str) -> Option<PathBuf> {
    if command.contains('/') {
        let path = PathBuf::from(command);
        return is_executable(&path).then_some(path);
    }
    let path_var = env::var_os("PATH")?;
    for directory in env::split_paths(&path_var) {
        let candidate = directory.join(command);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn has_windows_unsafe_component(path: &Path) -> bool {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    absolute.components().any(|part| match part {
        Component::Normal(value) => {
            let value = value.to_string_lossy();
            value.ends_with('.') || value.ends_with(' ')
        }
        _ => false,
    })
}

fn same_file(left: &Path, right: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match (fs::metadata(left), fs::metadata(right)) {
            (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
            _ => false,
        }
    }
    #[cfg(not(unix))]
    {
        left == right
    }
}

fn wine_compatible_base(path: &Path) -> PathBuf {
    if !has_windows_unsafe_component(path) {
        return path.to_path_buf();
    }
    let flatpak_data = Path::new("/var/data");
    let data_home = path.parent().unwrap_or(Path::new("/"));
    if env::var_os("FLATPAK_ID").is_some()
        && flatpak_data.is_dir()
        && same_file(data_home, flatpak_data)
    {
        if let Ok(relative) = path.strip_prefix(data_home) {
            return flatpak_data.join(relative);
        }
    }
    path.to_path_buf()
}

fn check_flatpak_extension(command: &str, path: &str) -> bool {
    if env::var_os("FLATPAK_ID").is_some() && !Path::new(path).exists() {
        return false;
    }
    which(command).is_some()
}

fn lsfg_vk_version() -> u8 {
    let mut directories = env::var("VK_ADD_LAYER_PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|item| !item.is_empty())
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    directories.extend(
        [
            "/usr/lib/extensions/vulkan/lsfgvk/share/vulkan/implicit_layer.d",
            "/usr/share/vulkan/implicit_layer.d",
            "/usr/local/share/vulkan/implicit_layer.d",
            "/etc/vulkan/implicit_layer.d",
        ]
        .iter()
        .map(PathBuf::from),
    );
    directories.insert(
        0,
        Paths::detect()
            .xdg_data_home
            .join("vulkan/implicit_layer.d"),
    );
    let manifest_v2 = "VkLayer_LSFGVK_frame_generation.json";
    let manifest_v1 = "VkLayer_LS_frame_generation.json";
    if directories
        .iter()
        .any(|directory| directory.join(manifest_v2).is_file())
    {
        2
    } else if directories
        .iter()
        .any(|directory| directory.join(manifest_v1).is_file())
    {
        1
    } else {
        0
    }
}

fn base_version() -> String {
    let path = Path::new("/app/manifest.json");
    let Ok(content) = fs::read_to_string(path) else {
        return String::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return String::new();
    };
    let version = value
        .get("base-version")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    version
        .strip_prefix("stable-")
        .unwrap_or(version)
        .to_owned()
}

impl ToolAvailability {
    pub fn detect() -> Self {
        let paths = Paths::detect();
        let version = lsfg_vk_version();
        Self {
            gamemode: which("gamemoderun").is_some(),
            gamescope: check_flatpak_extension(
                "gamescope",
                "/usr/lib/extensions/vulkan/gamescope/bin/gamescope",
            ),
            hdr_wsi: Path::new("/usr/lib/extensions/vulkan/HdrWsi/lib/libVkLayer_hdr_wsi.so")
                .exists(),
            vkbasalt: vkbasalt_available(&paths),
            lsfg_vk: version > 0,
            lsfg_vk_version: version,
            mangohud: check_flatpak_extension(
                "mangohud",
                "/usr/lib/extensions/vulkan/MangoHud/bin/mangohud",
            ),
            obs_vkcapture: check_flatpak_extension(
                "obs-vkcapture",
                "/usr/lib/extensions/vulkan/OBSVkCapture/bin/obs-vkcapture",
            ),
            vmtouch: which("vmtouch").is_some(),
            wine: which("wine").is_some(),
            vulkaninfo: which("vulkaninfo").is_some(),
            cabextract: which("cabextract").is_some(),
            p7zip: which("7z").is_some() || which("7za").is_some(),
            icoextract: which("icoextract").is_some(),
            base_version: base_version(),
        }
    }
}

fn vkbasalt_available(paths: &Paths) -> bool {
    [
        PathBuf::from("/usr/lib/extensions/vulkan/vkBasalt/etc/vkBasalt"),
        PathBuf::from("/usr/local"),
        PathBuf::from("/usr/share/vkBasalt"),
        paths
            .xdg_data_home
            .join("vulkan/implicit_layer.d/VkLayer_VkBasalt.json"),
    ]
    .iter()
    .any(|path| path.exists())
}

/// Return a path with characters that `pathvalidate(..., platform="universal")`
/// would reject.  Spaces are replaced by the caller where appropriate.
pub fn sanitize_universal_filename(input: &str) -> String {
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'];
    let cleaned: String = input
        .chars()
        .map(|character| {
            if forbidden.contains(&character) || character.is_control() {
                '_'
            } else {
                character
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|character: char| character == '.' || character == ' ');
    if trimmed.is_empty() {
        "_".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// Reject path traversal and names that cannot be represented as a bottle folder.
pub fn validate_bottle_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(io_error("bottle name", "Bottle name cannot be empty."));
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
        return Err(BottlesErrorConfig::invalid(
            "Bottle name contains a path separator.",
        ));
    }
    let sanitized = sanitize_universal_filename(&trimmed.replace(' ', "-"));
    if sanitized == "_" {
        return Err(BottlesErrorConfig::invalid("Bottle name is not valid."));
    }
    Ok(trimmed.to_owned())
}

// Small helper namespace to keep validation messages consistent without adding
// another public type.
struct BottlesErrorConfig;
impl BottlesErrorConfig {
    fn invalid(message: &str) -> crate::error::BottlesError {
        crate::error::BottlesError::InvalidInput(message.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_python_major_minor() {
        assert_eq!(APP_VERSION, "67.4");
    }

    #[test]
    fn paths_are_nested_under_base() {
        let paths = Paths::detect();
        assert_eq!(paths.bottles, paths.base.join("bottles"));
        assert_eq!(paths.runners, paths.base.join("runners"));
        assert_eq!(paths.library, paths.base.join("library.yml"));
    }

    #[test]
    fn sanitize_removes_universal_forbidden_characters() {
        assert_eq!(sanitize_universal_filename("a/b:c"), "a_b_c");
        assert_eq!(sanitize_universal_filename(" ... "), "_");
    }
}

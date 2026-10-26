use crate::error::Result;
use crate::paths::{is_official_package, Paths, ToolAvailability, APP_VERSION};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct HealthReport {
    pub official_package: bool,
    pub version: String,
    pub desktop: Option<String>,
    pub x11: bool,
    pub x11_port: String,
    pub wayland: bool,
    pub graphics: GraphicsReport,
    pub kernel_type: String,
    pub kernel_version: String,
    pub disk_total: u64,
    pub disk_free: u64,
    pub ram_total: String,
    pub ram_available: String,
    pub bottles_envs: BTreeMap<String, String>,
    pub tools: ToolAvailability,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphicsReport {
    pub vendors: Vec<String>,
    pub vulkan: bool,
}

pub struct HealthChecker;

impl HealthChecker {
    pub fn collect() -> HealthReport {
        let x11_port = x_display();
        let graphics = graphics_report();
        let (kernel_type, kernel_version) = kernel_info();
        let (disk_total, disk_free) = disk_usage();
        let (ram_total, ram_available) = ram_info();
        HealthReport {
            official_package: is_official_package(),
            version: APP_VERSION.to_owned(),
            desktop: std::env::var("DESKTOP_SESSION").ok(),
            x11: !x11_port.is_empty(),
            x11_port,
            wayland: std::env::var_os("WAYLAND_DISPLAY").is_some()
                || std::env::var_os("WAYLAND_SOCKET").is_some(),
            graphics,
            kernel_type,
            kernel_version,
            disk_total,
            disk_free,
            ram_total,
            ram_available,
            bottles_envs: bottles_environment(),
            tools: ToolAvailability::detect(),
        }
    }
}

impl HealthReport {
    /// Keys match `HealthChecker.get_results()` in the Python implementation.
    pub fn to_json(&self) -> Value {
        json!({
            "Official Package": self.official_package,
            "Version": self.version,
            "DE/WM": self.desktop,
            "Display": {
                "X.org": self.x11,
                "X.org (port)": self.x11_port,
                "Wayland": self.wayland,
            },
            "Graphics": {
                "vendors": self.graphics.vendors,
                "prime": {"integrated": Value::Null, "discrete": Value::Null},
                "Vulkan": self.graphics.vulkan,
            },
            "Kernel": {"Type": self.kernel_type, "Version": self.kernel_version},
            "Disk": {
                "Total": human_size(self.disk_total as f64),
                "Used": human_size((self.disk_total.saturating_sub(self.disk_free)) as f64),
                "Free": human_size(self.disk_free as f64),
            },
            "RAM": {"MemTotal": self.ram_total, "MemAvailable": self.ram_available},
            "Bottles_envs": self.bottles_envs,
            "Tools": self.tools,
        })
    }

    pub fn to_yaml(&self) -> Result<String> {
        let value = crate::config::json_value_to_yaml(&self.to_json());
        Ok(serde_yaml::to_string(&value)?.replace("&id", "&amp;id"))
    }
}

fn x_display() -> String {
    let display = std::env::var("DISPLAY").unwrap_or_default();
    if display.is_empty() {
        return String::new();
    }
    // Python's DisplayUtils returns the display port such as ":0".
    display
        .rsplit(':')
        .next()
        .map(|value| format!(":{value}"))
        .unwrap_or_default()
}

fn graphics_report() -> GraphicsReport {
    let mut vendors = Vec::new();
    if let Some(output) = command_stdout("lspci", &[]) {
        let lowercase = output.to_ascii_lowercase();
        for vendor in ["nvidia", "amd", "intel"] {
            if lowercase.contains(vendor) {
                vendors.push(vendor.to_owned());
            }
        }
    }
    let vulkan = ToolAvailability::detect().vulkaninfo
        && Command::new("vulkaninfo")
            .arg("--summary")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
    GraphicsReport { vendors, vulkan }
}

fn kernel_info() -> (String, String) {
    let output = command_stdout("uname", &["-sr"]).unwrap_or_default();
    let mut parts = output.split_whitespace();
    let kernel_type = parts.next().unwrap_or("Linux").to_owned();
    let kernel_version = parts.next().unwrap_or_default().to_owned();
    (kernel_type, kernel_version)
}

fn disk_usage() -> (u64, u64) {
    let output = command_stdout("df", &["-B1", "/"]).unwrap_or_default();
    let line = output.lines().nth(1).unwrap_or_default();
    let parts: Vec<&str> = line.split_whitespace().collect();
    let total = parts
        .get(1)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let free = parts
        .get(3)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    (total, free)
}

fn ram_info() -> (String, String) {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = "n/a".to_owned();
    let mut available = "n/a".to_owned();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next().unwrap_or_default();
        let value = parts
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .map(|value| human_size(value * 1024.0));
        match (key, value) {
            ("MemTotal:", Some(value)) => total = value,
            ("MemAvailable:", Some(value)) => available = value,
            _ => {}
        }
    }
    (total, available)
}

fn bottles_environment() -> BTreeMap<String, String> {
    for key in [
        "TESTING_REPOS",
        "LOCAL_INSTALLERS",
        "LOCAL_COMPONENTS",
        "LOCAL_DEPENDENCIES",
    ] {
        if let Ok(value) = std::env::var(key) {
            return BTreeMap::from([(key.to_owned(), value)]);
        }
    }
    BTreeMap::new()
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Mirror `FileUtils.get_human_size_legacy`.
pub fn human_size(mut size: f64) -> String {
    for unit in ["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"] {
        if size.abs() < 1024.0 {
            return format!("{size:3.1}{unit}B");
        }
        size /= 1024.0;
    }
    format!("{size:.1}YiB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_matches_python_legacy_units() {
        assert_eq!(human_size(512.0), "512.0B");
        assert_eq!(human_size(1024.0), "1.0KiB");
        assert_eq!(human_size(1024.0 * 1024.0), "1.0MiB");
    }

    #[test]
    fn health_json_uses_python_result_keys() {
        let report = HealthChecker::collect();
        let value = report.to_json();
        assert!(value.get("Official Package").is_some());
        assert!(value.get("Display").is_some());
        assert!(value.get("Graphics").is_some());
        assert!(value.get("Kernel").is_some());
        assert_eq!(value.get("Version").and_then(Value::as_str), Some("67.4"));
    }
}

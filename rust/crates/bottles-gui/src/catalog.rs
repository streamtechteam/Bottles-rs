use serde_json::Value;
use std::sync::OnceLock;

const CATALOGS_JSON: &str = include_str!("../assets/catalogs.json");

#[derive(Debug, Clone)]
pub struct CatalogSource {
    pub name: String,
    pub repository: String,
    pub commit: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Installer {
    pub id: String,
    pub name: String,
    pub description: String,
    pub grade: String,
    pub category: String,
    pub arch: String,
    pub channel: String,
    pub runners: Vec<String>,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub id: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub category: String,
    pub subcategory: String,
    pub channel: String,
    pub date: String,
    pub platforms: Vec<String>,
    pub architectures: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Catalogs {
    pub sources: Vec<CatalogSource>,
    pub installers: Vec<Installer>,
    pub dependencies: Vec<Dependency>,
    pub components: Vec<Component>,
}

static CATALOGS: OnceLock<Catalogs> = OnceLock::new();

pub fn catalogs() -> &'static Catalogs {
    CATALOGS.get_or_init(parse)
}

fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn parse() -> Catalogs {
    let root: Value = serde_json::from_str(CATALOGS_JSON).expect("catalogs.json is valid JSON");
    let mut sources = Vec::new();
    if let Some(map) = root.get("sources").and_then(Value::as_object) {
        for (name, source) in map {
            sources.push(CatalogSource {
                name: name.clone(),
                repository: string(source, "repository"),
                commit: string(source, "commit"),
                url: string(source, "url"),
            });
        }
    }
    sources.sort_by(|left, right| left.name.cmp(&right.name));

    let installers = root
        .get("installers")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|value| Installer {
            id: string(value, "id"),
            name: string(value, "name"),
            description: string(value, "description"),
            grade: string(value, "grade"),
            category: string(value, "category"),
            arch: string(value, "arch"),
            channel: string(value, "channel"),
            runners: strings(value, "runners"),
            icon: string(value, "icon"),
        })
        .collect();

    let dependencies = root
        .get("dependencies")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|value| Dependency {
            id: string(value, "id"),
            description: string(value, "description"),
            category: string(value, "category"),
        })
        .collect();

    let components = root
        .get("components")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|value| Component {
            id: string(value, "id"),
            category: string(value, "category"),
            subcategory: string(value, "subcategory"),
            channel: string(value, "channel"),
            date: string(value, "date"),
            platforms: strings(value, "platforms"),
            architectures: strings(value, "architectures"),
        })
        .collect();

    Catalogs {
        sources,
        installers,
        dependencies,
        components,
    }
}

impl Catalogs {
    /// Mirrors `InstallerManager.supports_channel` and
    /// `InstallerManager.supports_runner` from the Python implementation.
    pub fn installers_for(
        &self,
        arch: &str,
        runner: &str,
        include_unstable: bool,
    ) -> Vec<&Installer> {
        self.installers
            .iter()
            .filter(|installer| installer.arch == arch)
            .filter(|installer| include_unstable || installer.channel == "stable")
            .filter(|installer| {
                installer.runners.is_empty()
                    || installer.runners.iter().any(|candidate| {
                        runner == candidate || runner.starts_with(&format!("{candidate}-"))
                    })
            })
            .collect()
    }

    pub fn component_names(&self, category: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .components
            .iter()
            .filter(|component| component.category == category)
            .filter(|component| component.channel == "stable")
            .map(|component| component.id.clone())
            .collect();
        names.sort_by(|left, right| right.cmp(left));
        names
    }

    pub fn source(&self, name: &str) -> Option<&CatalogSource> {
        self.sources.iter().find(|source| source.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_are_loaded_from_pinned_sources() {
        let catalogs = catalogs();
        assert_eq!(catalogs.sources.len(), 3);
        assert_eq!(catalogs.installers.len(), 36);
        assert_eq!(catalogs.dependencies.len(), 126);
        assert_eq!(catalogs.components.len(), 800);
        assert!(catalogs
            .installers
            .iter()
            .any(|installer| installer.name == "Ableton Live"));
        assert!(catalogs.source("installers").is_some());
    }
}

use serde::Deserialize;
use std::collections::BTreeMap;

const CONFIG_FIELDS_JSON: &str = include_str!("../../../contracts/config_fields.json");
const SETTINGS_JSON: &str = include_str!("../../../contracts/settings.json");
const CLI_JSON: &str = include_str!("../../../contracts/cli.json");
const UI_JSON: &str = include_str!("../../../contracts/ui.json");
const MODULES_JSON: &str = include_str!("../../../contracts/modules.json");

#[derive(Debug, Clone, Deserialize)]
pub struct NamedField {
    pub name: String,
    #[serde(default)]
    pub python_type: String,
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub default_factory: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFieldsContract {
    pub source: String,
    pub bottle_config: Vec<NamedField>,
    pub bottle_params: Vec<NamedField>,
    pub bottle_sandbox_params: Vec<NamedField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingContract {
    pub name: String,
    #[serde(rename = "type")]
    pub gvariant_type: String,
    pub default: serde_json::Value,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsContract {
    pub source: String,
    pub keys: Vec<SettingContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliArgumentContract {
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub all_flags: Vec<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    #[serde(rename = "help")]
    pub help_text: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliCommandContract {
    pub name: String,
    #[serde(default)]
    #[serde(rename = "help")]
    pub help_text: String,
    #[serde(default)]
    pub arguments: Vec<CliArgumentContract>,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliContract {
    pub source: String,
    pub commands: Vec<CliCommandContract>,
    pub global_arguments: Vec<CliArgumentContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiScreenContract {
    pub name: String,
    pub source: String,
    pub kind: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub rust_surface: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiContract {
    pub source: String,
    pub screens: Vec<UiScreenContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleContract {
    pub name: String,
    pub source: String,
    pub area: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub rust_surface: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModulesContract {
    pub source: String,
    pub modules: Vec<ModuleContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contracts {
    pub config_fields: ConfigFieldsContract,
    pub settings: SettingsContract,
    pub cli: CliContract,
    pub ui: UiContract,
    pub modules: ModulesContract,
}

impl Contracts {
    fn load() -> Self {
        Self {
            config_fields: serde_json::from_str(CONFIG_FIELDS_JSON)
                .expect("embedded config_fields contract must be valid"),
            settings: serde_json::from_str(SETTINGS_JSON)
                .expect("embedded settings contract must be valid"),
            cli: serde_json::from_str(CLI_JSON).expect("embedded cli contract must be valid"),
            ui: serde_json::from_str(UI_JSON).expect("embedded ui contract must be valid"),
            modules: serde_json::from_str(MODULES_JSON)
                .expect("embedded modules contract must be valid"),
        }
    }
}

/// Return the source-derived contracts.  The value is cached after first use.
pub fn contracts() -> &'static Contracts {
    use std::sync::OnceLock;
    static CONTRACTS: OnceLock<Contracts> = OnceLock::new();
    CONTRACTS.get_or_init(Contracts::load)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParityStatus {
    Ported,
    Partial,
    InventoryOnly,
    ReferenceOnly,
}

impl ParityStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ported => "ported",
            Self::Partial => "partial",
            Self::InventoryOnly => "inventory-only",
            Self::ReferenceOnly => "reference-only",
        }
    }

    pub fn from_label(value: &str) -> Self {
        match value {
            "ported" => Self::Ported,
            "partial" => Self::Partial,
            "inventory-only" => Self::InventoryOnly,
            _ => Self::ReferenceOnly,
        }
    }

    pub const ALL: &'static [Self] = &[
        Self::Ported,
        Self::Partial,
        Self::InventoryOnly,
        Self::ReferenceOnly,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureRecord {
    pub kind: String,
    pub id: String,
    pub source: String,
    pub rust_surface: String,
    pub status: ParityStatus,
    pub notes: String,
}

impl FeatureRecord {
    pub fn matches(&self, search: &str) -> bool {
        let needle = search.trim().to_ascii_lowercase();
        needle.is_empty()
            || self.id.to_ascii_lowercase().contains(&needle)
            || self.source.to_ascii_lowercase().contains(&needle)
            || self.kind.to_ascii_lowercase().contains(&needle)
            || self.rust_surface.to_ascii_lowercase().contains(&needle)
            || self.status.label().contains(&needle)
    }
}

/// Build a complete, searchable feature inventory from the generated contracts.
pub fn inventory() -> Vec<FeatureRecord> {
    let contracts = contracts();
    let mut records = Vec::new();

    for group in [
        (
            "config",
            &contracts.config_fields.bottle_config,
            "crates/bottles-core/src/config.rs",
        ),
        (
            "parameter",
            &contracts.config_fields.bottle_params,
            "crates/bottles-core/src/config.rs",
        ),
        (
            "sandbox-parameter",
            &contracts.config_fields.bottle_sandbox_params,
            "crates/bottles-core/src/config.rs",
        ),
    ] {
        for field in group.1 {
            records.push(FeatureRecord {
                kind: group.0.to_owned(),
                id: field.name.clone(),
                source: contracts.config_fields.source.clone(),
                rust_surface: group.2.to_owned(),
                status: ParityStatus::Ported,
                notes: format!("Python type: {}", field.python_type),
            });
        }
    }

    for setting in &contracts.settings.keys {
        records.push(FeatureRecord {
            kind: "setting".to_owned(),
            id: setting.name.clone(),
            source: contracts.settings.source.clone(),
            rust_surface: "crates/bottles-core/src/settings.rs".to_owned(),
            status: ParityStatus::Ported,
            notes: format!("GSettings type: {}", setting.gvariant_type),
        });
    }

    for command in &contracts.cli.commands {
        records.push(FeatureRecord {
            kind: "cli-command".to_owned(),
            id: command.name.clone(),
            source: contracts.cli.source.clone(),
            rust_surface: "crates/bottles-cli/src/main.rs".to_owned(),
            status: ParityStatus::from_label(&command.status),
            notes: command.help_text.clone(),
        });
        for argument in &command.arguments {
            let flags = argument.all_flags.join(" / ");
            let id = format!("{} {}", command.name, flags);
            records.push(FeatureRecord {
                kind: "cli-argument".to_owned(),
                id,
                source: contracts.cli.source.clone(),
                rust_surface: "crates/bottles-cli/src/main.rs".to_owned(),
                status: ParityStatus::from_label(&command.status),
                notes: argument.help_text.clone(),
            });
        }
    }

    for screen in &contracts.ui.screens {
        records.push(FeatureRecord {
            kind: if screen.kind == "dialog" {
                "ui-dialog".to_owned()
            } else {
                "ui-screen".to_owned()
            },
            id: screen.name.clone(),
            source: screen.source.clone(),
            rust_surface: screen.rust_surface.clone(),
            status: ParityStatus::from_label(&screen.status),
            notes: screen.kind.clone(),
        });
    }

    for module in &contracts.modules.modules {
        records.push(FeatureRecord {
            kind: "python-module".to_owned(),
            id: module.source.clone(),
            source: module.source.clone(),
            rust_surface: module.rust_surface.clone(),
            status: ParityStatus::from_label(&module.status),
            notes: module.area.clone(),
        });
    }

    records
}

pub fn summary() -> BTreeMap<&'static str, usize> {
    let mut result = BTreeMap::new();
    for status in ParityStatus::ALL {
        result.insert(status.label(), 0);
    }
    for record in inventory() {
        *result.get_mut(record.status.label()).unwrap() += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contracts_are_source_derived_and_nonempty() {
        let contracts = contracts();
        assert_eq!(contracts.config_fields.bottle_config.len(), 35);
        assert_eq!(contracts.config_fields.bottle_params.len(), 56);
        assert_eq!(contracts.config_fields.bottle_sandbox_params.len(), 4);
        assert_eq!(contracts.settings.keys.len(), 33);
        assert_eq!(contracts.cli.commands.len(), 15);
        assert_eq!(contracts.ui.screens.len(), 78);
        assert!(!contracts.modules.modules.is_empty());
    }

    #[test]
    fn inventory_has_no_duplicate_ids() {
        let mut ids = inventory()
            .into_iter()
            .map(|record| format!("{}:{}", record.kind, record.id))
            .collect::<Vec<_>>();
        let original = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original);
    }

    #[test]
    fn inventory_contains_every_generated_cli_command() {
        let inventory = inventory();
        for command in &contracts().cli.commands {
            assert!(inventory
                .iter()
                .any(|record| { record.kind == "cli-command" && record.id == command.name }));
        }
    }
}

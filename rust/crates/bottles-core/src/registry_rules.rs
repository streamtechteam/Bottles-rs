use crate::config::{BottleConfig, RegistryRule};
use crate::error::{BottlesError, IoResultExt, Result};
use crate::paths::Paths;
use crate::wine::{WineLauncher, WineTool};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub struct RegistryRuleManager;

impl RegistryRuleManager {
    pub fn load_rules(config: &BottleConfig) -> BTreeMap<String, RegistryRule> {
        config
            .registry_rules
            .iter()
            .filter(|rule| !rule.name.is_empty())
            .map(|rule| (rule.name.clone(), rule.clone()))
            .collect()
    }

    pub fn list_rules(config: &BottleConfig) -> Vec<RegistryRule> {
        Self::load_rules(config).into_values().collect()
    }

    pub fn upsert_rule(config: &mut BottleConfig, rule: RegistryRule) {
        let mut rules = Self::load_rules(config);
        rules.insert(rule.name.clone(), rule);
        config.registry_rules = rules.into_values().collect();
    }

    pub fn delete_rule(config: &mut BottleConfig, name: &str) -> bool {
        let mut rules = Self::load_rules(config);
        let removed = rules.remove(name).is_some();
        if removed {
            config.registry_rules = rules.into_values().collect();
        }
        removed
    }

    /// Equivalent to `RegistryRuleManager.apply_rules`: selected rules are
    /// written to temporary `.reg` bundles and imported with Wine's `reg`.
    pub fn apply_rules(
        paths: &Paths,
        config: &BottleConfig,
        prefix: &Path,
        rule_names: Option<&[String]>,
        trigger: Option<&str>,
    ) -> Result<Vec<String>> {
        let rules = Self::load_rules(config);
        let selected: Vec<RegistryRule> = match rule_names {
            Some(names) if !names.is_empty() => names
                .iter()
                .filter_map(|name| rules.get(name).cloned())
                .collect(),
            _ => rules.into_values().collect(),
        }
        .into_iter()
        .filter(|rule| match trigger {
            Some(trigger) => {
                rule.triggers.is_empty()
                    || rule
                        .triggers
                        .iter()
                        .any(|value| value == trigger || value == "all")
            }
            None => true,
        })
        .collect();

        fs::create_dir_all(&paths.temp).with_path(&paths.temp)?;
        let launcher = WineLauncher::new(paths.clone(), config.clone(), prefix.to_path_buf());
        let mut applied = Vec::new();
        for rule in selected {
            if rule.keys.trim().is_empty() {
                continue;
            }
            let keys = rule.keys.trim_start();
            let has_header = keys.to_ascii_uppercase().starts_with("REGEDIT4")
                || keys
                    .to_ascii_lowercase()
                    .starts_with("windows registry editor version");
            let mut content = String::new();
            if !has_header {
                content.push_str("REGEDIT4\n\n");
            }
            content.push_str(keys.trim_end());
            content.push('\n');
            let file = paths.temp.join(format!("{}.reg", uuid::Uuid::new_v4()));
            fs::write(&file, content).with_path(&file)?;
            let plan = launcher.tool_plan(
                WineTool::Reg,
                &["import".to_owned(), file.to_string_lossy().into_owned()],
            )?;
            let output = launcher.execute(&plan)?;
            let _ = fs::remove_file(&file);
            if !output.status.success() {
                return Err(BottlesError::CommandFailed {
                    command: format!("reg import {}", rule.name),
                    message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
                });
            }
            applied.push(rule.name);
        }
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BottleConfig;

    #[test]
    fn upsert_and_delete_are_keyed_by_name() {
        let mut config = BottleConfig::default();
        RegistryRuleManager::upsert_rule(
            &mut config,
            RegistryRule {
                name: "fsr".to_owned(),
                description: "FSR".to_owned(),
                keys: "REGEDIT4".to_owned(),
                triggers: vec!["runner_change".to_owned()],
                run_once: false,
            },
        );
        assert_eq!(RegistryRuleManager::list_rules(&config).len(), 1);
        RegistryRuleManager::upsert_rule(
            &mut config,
            RegistryRule {
                name: "fsr".to_owned(),
                description: "Updated".to_owned(),
                ..RegistryRule::default()
            },
        );
        assert_eq!(RegistryRuleManager::list_rules(&config).len(), 1);
        assert_eq!(
            RegistryRuleManager::list_rules(&config)[0].description,
            "Updated"
        );
        assert!(RegistryRuleManager::delete_rule(&mut config, "fsr"));
        assert!(RegistryRuleManager::list_rules(&config).is_empty());
    }
}

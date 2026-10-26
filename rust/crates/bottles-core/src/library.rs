use crate::error::{IoResultExt, Result};
use crate::paths::Paths;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const LOCAL_COVER_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".webp"];

/// A compatibility layer for `library.yml`.  Entries are kept as YAML mappings
/// so metadata used by Python features remains intact.
#[derive(Debug, Clone)]
pub struct Library {
    pub path: PathBuf,
    pub entries: BTreeMap<String, Value>,
}

impl Library {
    pub fn load(paths: &Paths) -> Result<Self> {
        let mut library = Self {
            path: paths.library.clone(),
            entries: BTreeMap::new(),
        };
        library.reload()?;
        Ok(library)
    }

    pub fn reload(&mut self) -> Result<()> {
        if !self.path.exists() {
            self.entries = BTreeMap::new();
            self.save()?;
            return Ok(());
        }
        let content = fs::read_to_string(&self.path).with_path(&self.path)?;
        let value: Value = serde_yaml::from_str(&content)?;
        self.entries = BTreeMap::new();
        if let Value::Mapping(mapping) = value {
            for (key, value) in mapping {
                let Some(id) = key.as_str() else { continue };
                // Python drops any entry without an `id` field.
                if value
                    .as_mapping()
                    .map(|entry| entry.contains_key(&Value::String("id".to_owned())))
                    .unwrap_or(false)
                {
                    self.entries.insert(id.to_owned(), value);
                }
            }
        }
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).with_path(parent)?;
        let content = serde_yaml::to_string(&self.entries)?;
        fs::write(&self.path, content).with_path(&self.path)
    }

    pub fn entry(&self, id: &str) -> Option<&Value> {
        self.entries.get(id)
    }

    pub fn entry_field(&self, id: &str, field: &str) -> Option<&Value> {
        self.entry(id)?
            .as_mapping()?
            .get(&Value::String(field.to_owned()))
    }

    pub fn name(&self, id: &str) -> String {
        self.entry_field(id, "name")
            .and_then(Value::as_str)
            .unwrap_or("Untitled")
            .to_owned()
    }

    pub fn add(&mut self, entry: Mapping) -> Result<String> {
        let id_value = entry
            .get(&Value::String("id".to_owned()))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        if id_value.is_empty() {
            return Err(crate::error::BottlesError::InvalidInput(
                "library entries require an id".to_owned(),
            ));
        }
        if self.entries.values().any(|value| {
            value
                .as_mapping()
                .and_then(|mapping| mapping.get(&Value::String("id".to_owned())))
                .and_then(Value::as_str)
                == Some(id_value.as_str())
        }) {
            return Err(crate::error::BottlesError::InvalidInput(format!(
                "entry `{id_value}` is already in the library"
            )));
        }
        let uuid = uuid::Uuid::new_v4().to_string();
        self.entries.insert(uuid.clone(), Value::Mapping(entry));
        self.save()?;
        Ok(uuid)
    }

    pub fn remove(&mut self, uuid: &str) -> Result<bool> {
        let removed = self.entries.remove(uuid).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn rename_bottle(&mut self, old_name: &str, new_name: &str) -> Result<usize> {
        let mut changed = 0;
        for entry in self.entries.values_mut() {
            let Some(mapping) = entry.as_mapping_mut() else {
                continue;
            };
            let bottle_key = Value::String("bottle".to_owned());
            let Some(Value::Mapping(bottle)) = mapping.get_mut(&bottle_key) else {
                continue;
            };
            let name_key = Value::String("name".to_owned());
            if bottle.get(&name_key).and_then(Value::as_str) == Some(old_name) {
                bottle.insert(name_key, Value::String(new_name.to_owned()));
                changed += 1;
            }
        }
        if changed > 0 {
            self.save()?;
        }
        Ok(changed)
    }

    /// Mirror `LibraryManager.sync_umu_game`.
    pub fn sync_umu_game(
        &mut self,
        game_id: &str,
        source_id: &str,
        name: &str,
        store: &str,
    ) -> Result<()> {
        let library_id = format!("umu:{game_id}");
        let data = vec![
            ("id", library_id.clone()),
            ("source", "umu".to_owned()),
            ("source_id", source_id.to_owned()),
            ("name", name.to_owned()),
            ("store", store.to_owned()),
        ];
        for entry in self.entries.values_mut() {
            let Some(mapping) = entry.as_mapping_mut() else {
                continue;
            };
            if mapping
                .get(&Value::String("id".to_owned()))
                .and_then(Value::as_str)
                == Some(library_id.as_str())
            {
                for (key, value) in data {
                    mapping.insert(Value::String(key.to_owned()), Value::String(value));
                }
                return self.save();
            }
        }
        let mut mapping = Mapping::new();
        for (key, value) in data {
            mapping.insert(Value::String(key.to_owned()), Value::String(value));
        }
        self.add(mapping)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(name: &str) -> Paths {
        let mut paths = Paths::detect();
        paths.base =
            std::env::temp_dir().join(format!("bottles-rs-{name}-{}", uuid::Uuid::new_v4()));
        paths.library = paths.base.join("library.yml");
        paths
    }

    #[test]
    fn library_drops_entries_without_id_and_persists_valid_entries() {
        let paths = temp_paths("library");
        let mut library = Library::load(&paths).unwrap();
        let mut valid = Mapping::new();
        valid.insert(
            Value::String("id".to_owned()),
            Value::String("prog-1".to_owned()),
        );
        valid.insert(
            Value::String("name".to_owned()),
            Value::String("Game".to_owned()),
        );
        let uuid = library.add(valid).unwrap();
        let mut invalid = Mapping::new();
        invalid.insert(
            Value::String("id".to_owned()),
            Value::String("broken".to_owned()),
        );
        library
            .entries
            .insert("orphan".to_owned(), Value::Mapping(invalid));
        // Remove id to emulate the Python cleanup rule.
        library
            .entries
            .get_mut("orphan")
            .unwrap()
            .as_mapping_mut()
            .unwrap()
            .remove(&Value::String("id".to_owned()));
        library.save().unwrap();
        library.reload().unwrap();
        assert!(library.entry(&uuid).is_some());
        assert!(library.entry("orphan").is_none());
        fs::remove_dir_all(paths.base).unwrap();
    }

    #[test]
    fn rename_bottle_updates_library_metadata() {
        let paths = temp_paths("rename");
        let mut library = Library::load(&paths).unwrap();
        let mut bottle = Mapping::new();
        bottle.insert(
            Value::String("name".to_owned()),
            Value::String("Old".to_owned()),
        );
        let mut entry = Mapping::new();
        entry.insert(
            Value::String("id".to_owned()),
            Value::String("game".to_owned()),
        );
        entry.insert(Value::String("bottle".to_owned()), Value::Mapping(bottle));
        library.add(entry).unwrap();
        assert_eq!(library.rename_bottle("Old", "New").unwrap(), 1);
        let key = library.entries.keys().next().unwrap().clone();
        let mapping = library.entry(&key).unwrap().as_mapping().unwrap();
        let bottle = mapping
            .get(&Value::String("bottle".to_owned()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            bottle.get(&Value::String("name".to_owned())),
            Some(&Value::String("New".to_owned()))
        );
        fs::remove_dir_all(paths.base).unwrap();
    }
}

use crate::error::{BottlesError, IoResultExt, Result};
use crate::paths::Paths;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const JOURNAL_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const BACKUP_TIMESTAMP_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JournalSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
    Crash,
}

impl JournalSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Crash => "crash",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalPeriod {
    All,
    Today,
    Yesterday,
    Week,
    Month,
}

impl JournalPeriod {
    pub fn from_str(value: &str) -> Self {
        match value {
            "all" => Self::All,
            "yesterday" => Self::Yesterday,
            "week" => Self::Week,
            "month" => Self::Month,
            _ => Self::Today,
        }
    }

    pub const ALL: &'static [&'static str] = &["all", "today", "yesterday", "week", "month"];
}

#[derive(Debug, Clone)]
pub struct Journal {
    pub path: PathBuf,
}

impl Journal {
    pub fn load(paths: &Paths) -> Result<Self> {
        let journal = Self {
            path: paths.journal.clone(),
        };
        journal.load_entries()?;
        Ok(journal)
    }

    /// `JournalManager.__get_journal`: create, back up malformed files, and
    /// return entries sorted by descending timestamp.
    pub fn load_entries(&self) -> Result<BTreeMap<String, Value>> {
        if !self.path.exists() {
            self.save_entries(&BTreeMap::new())?;
        }
        let content = fs::read_to_string(&self.path).with_path(&self.path)?;
        let value: Value = match serde_yaml::from_str(&content) {
            Ok(value) => value,
            Err(_) => {
                let backup = self.path.with_file_name(format!(
                    "{}_{}.bak",
                    self.path.file_name().unwrap_or_default().to_string_lossy(),
                    Local::now().format(BACKUP_TIMESTAMP_FORMAT)
                ));
                let _ = fs::copy(&self.path, &backup);
                Value::Mapping(serde_yaml::Mapping::new())
            }
        };
        let mut entries = BTreeMap::new();
        if let Value::Mapping(mapping) = value {
            for (key, value) in mapping {
                if let Some(id) = key.as_str() {
                    entries.insert(id.to_owned(), value);
                }
            }
        }
        let mut sorted: Vec<(String, Value, NaiveDateTime)> = entries
            .into_iter()
            .filter_map(|(id, value)| {
                let timestamp = value
                    .as_mapping()?
                    .get(&Value::String("timestamp".to_owned()))?
                    .as_str()?;
                let parsed = NaiveDateTime::parse_from_str(timestamp, JOURNAL_TIMESTAMP_FORMAT).ok()?;
                Some((id, value, parsed))
            })
            .collect();
        sorted.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
        Ok(sorted.into_iter().map(|(id, value, _)| (id, value)).collect())
    }

    pub fn save_entries(&self, entries: &BTreeMap<String, Value>) -> Result<()> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).with_path(parent)?;
        let content = serde_yaml::to_string(entries)?;
        fs::write(&self.path, content).with_path(&self.path)
    }

    pub fn write(&self, severity: JournalSeverity, message: &str) -> Result<String> {
        let mut entries = self.load_entries()?;
        let id = uuid::Uuid::new_v4().to_string();
        let mut entry = serde_yaml::Mapping::new();
        entry.insert(
            Value::String("severity".to_owned()),
            Value::String(severity.as_str().to_owned()),
        );
        entry.insert(
            Value::String("message".to_owned()),
            Value::String(message.to_owned()),
        );
        entry.insert(
            Value::String("timestamp".to_owned()),
            Value::String(
                Local::now()
                    .naive_local()
                    .format(JOURNAL_TIMESTAMP_FORMAT)
                    .to_string(),
            ),
        );
        entries.insert(id.clone(), Value::Mapping(entry));
        self.save_entries(&entries)?;
        self.clean_old()?;
        Ok(id)
    }

    /// `JournalManager.__clean_old`: remove events older than 30 days.
    pub fn clean_old(&self) -> Result<usize> {
        let mut entries = self.load_entries()?;
        let cutoff = Local::now().naive_local() - Duration::days(30);
        let old: Vec<String> = entries
            .iter()
            .filter_map(|(id, value)| {
                let timestamp = value
                    .as_mapping()?
                    .get(&Value::String("timestamp".to_owned()))?
                    .as_str()?;
                let parsed = NaiveDateTime::parse_from_str(timestamp, JOURNAL_TIMESTAMP_FORMAT).ok()?;
                (parsed < cutoff).then(|| id.clone())
            })
            .collect();
        for id in &old {
            entries.remove(id);
        }
        if !old.is_empty() {
            self.save_entries(&entries)?;
        }
        Ok(old.len())
    }

    pub fn get(&self, period: JournalPeriod) -> Result<BTreeMap<String, Value>> {
        let entries = self.load_entries()?;
        if period == JournalPeriod::All {
            return Ok(entries);
        }
        let today = Local::now().date_naive();
        let (start, end) = match period {
            JournalPeriod::All => (NaiveDate::MIN, NaiveDate::MAX),
            JournalPeriod::Today => (today, today + Duration::days(1)),
            JournalPeriod::Yesterday => (today - Duration::days(1), today),
            JournalPeriod::Week => (today - Duration::days(7), today + Duration::days(1)),
            JournalPeriod::Month => (today - Duration::days(30), today + Duration::days(1)),
        };
        Ok(entries
            .into_iter()
            .filter(|(_, value)| {
                value
                    .as_mapping()
                    .and_then(|mapping| mapping.get(&Value::String("timestamp".to_owned())))
                    .and_then(Value::as_str)
                    .and_then(|value| {
                        NaiveDateTime::parse_from_str(value, JOURNAL_TIMESTAMP_FORMAT).ok()
                    })
                    .map(|timestamp| {
                        let date = timestamp.date();
                        date >= start && date <= end
                    })
                    .unwrap_or(false)
            })
            .collect())
    }

    pub fn event(&self, id: &str) -> Result<Option<Value>> {
        Ok(self.load_entries()?.get(id).cloned())
    }

    pub fn first_event_date(&self) -> Result<Option<NaiveDateTime>> {
        let mut earliest = None;
        for value in self.load_entries()?.values() {
            let Some(timestamp) = value
                .as_mapping()
                .and_then(|mapping| mapping.get(&Value::String("timestamp".to_owned())))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let Ok(parsed) = NaiveDateTime::parse_from_str(timestamp, JOURNAL_TIMESTAMP_FORMAT) else {
                continue;
            };
            if earliest.map(|current: NaiveDateTime| parsed < current).unwrap_or(true) {
                earliest = Some(parsed);
            }
        }
        Ok(earliest)
    }

    pub fn plain(&self, period: JournalPeriod) -> Result<String> {
        let entries = self.get(period)?;
        let mut value = serde_yaml::to_value(entries)?;
        // Replace &id anchors the same way the Python crash-report code does.
        if let Some(mapping) = value.as_mapping_mut() {
            for (_, entry) in mapping.iter_mut() {
                if let Some(message) = entry
                    .as_mapping_mut()
                    .and_then(|entry| entry.get_mut(&Value::String("message".to_owned())))
                {
                    if let Some(text) = message.as_str() {
                        *message = Value::String(text.replace("&id", "&amp;id"));
                    }
                }
            }
        }
        Ok(serde_yaml::to_string(&value).map_err(BottlesError::from)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_journal(name: &str) -> (Paths, Journal) {
        let mut paths = Paths::detect();
        paths.base = std::env::temp_dir().join(format!("bottles-rs-{name}-{}", uuid::Uuid::new_v4()));
        paths.journal = paths.base.join("journal.yml");
        let journal = Journal::load(&paths).unwrap();
        (paths, journal)
    }

    #[test]
    fn journal_creates_and_filters_events() {
        let (paths, journal) = temp_journal("journal");
        let id = journal.write(JournalSeverity::Info, "started").unwrap();
        assert!(journal.event(&id).unwrap().is_some());
        assert_eq!(journal.get(JournalPeriod::Today).unwrap().len(), 1);
        assert_eq!(journal.get(JournalPeriod::All).unwrap().len(), 1);
        fs::remove_dir_all(paths.base).unwrap();
    }

    #[test]
    fn malformed_journal_is_backed_up() {
        let (paths, journal) = temp_journal("malformed");
        fs::create_dir_all(&paths.base).unwrap();
        fs::write(&journal.path, "not: [valid").unwrap();
        assert!(journal.load_entries().unwrap().is_empty());
        let backups = fs::read_dir(&paths.base)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bak"))
            .count();
        assert_eq!(backups, 1);
        fs::remove_dir_all(paths.base).unwrap();
    }
}

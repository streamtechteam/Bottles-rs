use crate::error::{BottlesError, IoResultExt, Result};
use crate::paths::{which, Paths};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: i64 = 1;
pub const DEFAULT_PROTON_VALUE: &str = "ProtoSoda";
pub const UMU_STORE_IDS: &[&str] = &[
    "none",
    "amazon",
    "battlenet",
    "ea",
    "egs",
    "gog",
    "humble",
    "itchio",
    "steam",
    "ubisoft",
    "umu",
    "zoomplatform",
];
pub const GAME_STATES: &[&str] = &["draft", "installing", "ready", "failed", "stopped"];

pub const RESERVED_ENVIRONMENT_KEYS: &[&str] = &[
    "EXE",
    "GAMEID",
    "PROTONPATH",
    "PROTON_VERB",
    "RUNTIMEPATH",
    "SODA_ADAPTIVE_PROFILE",
    "SODA_ADAPTIVE_TRACE_DIR",
    "STEAM_COMPAT_APP_ID",
    "STEAM_COMPAT_DATA_PATH",
    "STEAM_COMPAT_INSTALL_PATH",
    "STEAM_COMPAT_MOUNTS",
    "STEAM_COMPAT_SHADER_PATH",
    "STEAM_COMPAT_TOOL_PATHS",
    "STORE",
    "SteamAppId",
    "SteamGameId",
    "UMU_ID",
    "UMU_NO_RUNTIME",
    "WINEPREFIX",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UmuPrefix {
    pub path: String,
    pub managed: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for UmuPrefix {
    fn default() -> Self {
        Self {
            path: String::new(),
            managed: true,
            extra: BTreeMap::new(),
        }
    }
}

impl UmuPrefix {
    pub fn validate(&self) -> Result<()> {
        clean_text(&self.path, "prefix path", false)?;
        if self.managed && Path::new(&self.path).is_absolute() {
            return Err(BottlesError::InvalidInput(
                "Managed prefix paths must be relative".to_owned(),
            ));
        }
        if !self.managed && !Path::new(&self.path).is_absolute() {
            return Err(BottlesError::InvalidInput(
                "Custom prefix paths must be absolute".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn resolve(&self, data_root: &Path) -> Result<PathBuf> {
        self.validate()?;
        let path = PathBuf::from(&self.path);
        if !self.managed {
            return Ok(normalize_absolute(&path));
        }
        let root = normalize_absolute(data_root);
        let resolved = normalize_absolute(&root.join(path));
        if !resolved.starts_with(&root) {
            return Err(BottlesError::InvalidInput(
                "Managed prefix path escapes the UMU data root".to_owned(),
            ));
        }
        Ok(resolved)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UmuGame {
    pub schema_version: i64,
    pub id: Uuid,
    pub name: String,
    pub executable: PathBuf,
    pub prefix: UmuPrefix,
    pub proton: String,
    pub state: String,
    pub game_id: String,
    pub store: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub environment: BTreeMap<String, String>,
    pub sandbox: bool,
    pub share_net: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for UmuGame {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            name: String::new(),
            executable: PathBuf::new(),
            prefix: UmuPrefix::default(),
            proton: DEFAULT_PROTON_VALUE.to_owned(),
            state: "draft".to_owned(),
            game_id: "umu-default".to_owned(),
            store: "none".to_owned(),
            arguments: Vec::new(),
            working_directory: None,
            environment: BTreeMap::new(),
            sandbox: false,
            share_net: false,
            extra: BTreeMap::new(),
        }
    }
}

impl UmuGame {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(BottlesError::InvalidInput(format!(
                "Unsupported UMU game schema version: {}",
                self.schema_version
            )));
        }
        clean_text(&self.name, "game name", false)?;
        clean_text(&self.proton, "Proton path", false)?;
        clean_text(&self.game_id, "UMU game id", false)?;
        if !GAME_STATES.contains(&self.state.as_str()) {
            return Err(BottlesError::InvalidInput("Invalid game state".to_owned()));
        }
        let store = clean_text(&self.store, "store", false)?;
        if !UMU_STORE_IDS.contains(&store.as_str()) {
            return Err(BottlesError::InvalidInput("Invalid store".to_owned()));
        }
        if !self.executable.is_absolute() {
            return Err(BottlesError::InvalidInput(
                "Executable path must be absolute".to_owned(),
            ));
        }
        if let Some(directory) = &self.working_directory {
            if !directory.is_absolute() {
                return Err(BottlesError::InvalidInput(
                    "Working directory path must be absolute".to_owned(),
                ));
            }
        }
        for (key, value) in &self.environment {
            clean_text(key, "environment key", false)?;
            if key.contains('=') {
                return Err(BottlesError::InvalidInput(
                    "Invalid environment key".to_owned(),
                ));
            }
            clean_text(value, "environment value", true)?;
        }
        self.prefix.validate()
    }

    pub fn library_id(&self) -> String {
        format!("umu:{}", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct UmuRepository {
    pub root: PathBuf,
    pub games_root: PathBuf,
    pub prefixes_root: PathBuf,
}

impl UmuRepository {
    pub fn new(root: Option<PathBuf>, paths: &Paths) -> Self {
        let root = root.unwrap_or_else(|| paths.umu.clone());
        Self {
            games_root: root.join("games"),
            prefixes_root: root.join("prefixes"),
            root,
        }
    }

    pub fn config_path(&self, id: Uuid) -> PathBuf {
        self.games_root.join(id.to_string()).join("game.yml")
    }

    pub fn prefix_path(&self, game: &UmuGame) -> Result<PathBuf> {
        game.prefix.resolve(&self.root)
    }

    pub fn new_game(
        &self,
        name: &str,
        executable: PathBuf,
        proton: String,
        game_id: String,
        store: String,
        sandbox: bool,
        share_net: bool,
    ) -> UmuGame {
        let id = Uuid::new_v4();
        UmuGame {
            id,
            name: name.to_owned(),
            executable,
            prefix: UmuPrefix {
                path: format!("prefixes/{id}"),
                managed: true,
                extra: BTreeMap::new(),
            },
            proton,
            game_id,
            store,
            sandbox,
            share_net,
            ..UmuGame::default()
        }
    }

    pub fn save(&self, game: &UmuGame) -> Result<PathBuf> {
        game.validate()?;
        let expected_prefix = self.prefixes_root.join(game.id.to_string());
        if game.prefix.managed && self.prefix_path(game)? != expected_prefix {
            return Err(BottlesError::InvalidInput(
                "Managed prefix does not belong to the UMU game".to_owned(),
            ));
        }
        let path = self.config_path(game.id);
        let parent = path.parent().unwrap_or(&self.games_root);
        fs::create_dir_all(parent).with_path(parent)?;
        let temporary = parent.join(format!(".game-{}.tmp", Uuid::new_v4()));
        let content = serde_yaml::to_string(game)?;
        fs::write(&temporary, content).with_path(&temporary)?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::remove_file(&temporary);
            return Err(BottlesError::Io {
                path: path.clone(),
                source: error,
            });
        }
        Ok(path)
    }

    pub fn load(&self, id: Uuid) -> Result<UmuGame> {
        let path = self.config_path(id);
        let content = fs::read_to_string(&path).with_path(&path)?;
        let game: UmuGame = serde_yaml::from_str(&content)?;
        game.validate()?;
        if game.id != id {
            return Err(BottlesError::InvalidInput(
                "UMU game id does not match its directory".to_owned(),
            ));
        }
        Ok(game)
    }

    pub fn list_games(&self) -> Result<Vec<UmuGame>> {
        if !self.games_root.is_dir() {
            return Ok(Vec::new());
        }
        let mut games = Vec::new();
        let mut directories = fs::read_dir(&self.games_root)
            .with_path(&self.games_root)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        directories.sort();
        for directory in directories {
            let Some(name) = directory.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let Ok(id) = Uuid::parse_str(name) else {
                continue;
            };
            if let Ok(game) = self.load(id) {
                games.push(game);
            }
        }
        Ok(games)
    }

    pub fn find(&self, query: &str) -> Result<Vec<UmuGame>> {
        Ok(self
            .list_games()?
            .into_iter()
            .filter(|game| {
                game.name == query
                    || game.id.to_string() == query
                    || game.game_id == query
                    || game.library_id() == query
            })
            .collect())
    }

    pub fn set_executable(&self, id: Uuid, executable: PathBuf) -> Result<UmuGame> {
        let mut game = self.load(id)?;
        game.executable = executable;
        game.validate()?;
        self.save(&game)?;
        Ok(game)
    }

    pub fn set_state(&self, id: Uuid, state: &str) -> Result<UmuGame> {
        if !GAME_STATES.contains(&state) {
            return Err(BottlesError::InvalidInput("Invalid game state".to_owned()));
        }
        let mut game = self.load(id)?;
        game.state = state.to_owned();
        self.save(&game)?;
        Ok(game)
    }

    pub fn delete(&self, id: Uuid) -> Result<bool> {
        let path = self.config_path(id);
        let Some(directory) = path.parent() else {
            return Ok(false);
        };
        if !directory.is_dir() {
            return Ok(false);
        }
        let tombstone = self.games_root.join(format!(".deleted-{}", Uuid::new_v4()));
        fs::rename(directory, &tombstone).with_path(directory)?;
        fs::remove_dir_all(&tombstone).with_path(&tombstone)?;
        Ok(true)
    }

    /// Prepare an `umu-run` command using the environment keys from the Python
    /// `UmuExecutor`.  Sandboxing and Proton catalog resolution remain partial.
    pub fn launch_command(
        &self,
        game: &UmuGame,
        launcher_override: Option<&str>,
        install_only: bool,
    ) -> Result<Command> {
        game.validate()?;
        let reserved: HashSet<&str> = RESERVED_ENVIRONMENT_KEYS.iter().copied().collect();
        if let Some(key) = game
            .environment
            .keys()
            .find(|key| reserved.contains(key.as_str()))
        {
            return Err(BottlesError::InvalidInput(format!(
                "Reserved UMU environment variable cannot be overridden: {key}"
            )));
        }
        let launcher = launcher_override
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| which("umu-run"))
            .ok_or_else(|| BottlesError::CommandFailed {
                command: "umu-run".to_owned(),
                message: "UMU launcher not found".to_owned(),
            })?;
        let prefix = self.prefix_path(game)?;
        fs::create_dir_all(&prefix).with_path(&prefix)?;
        let mut command = Command::new(launcher);
        command.arg(if install_only {
            String::new()
        } else {
            game.executable.to_string_lossy().into_owned()
        });
        if !install_only {
            command.args(&game.arguments);
            let install_path = game.working_directory.clone().unwrap_or_else(|| {
                game.executable
                    .parent()
                    .unwrap_or(Path::new("/"))
                    .to_path_buf()
            });
            command.env("STEAM_COMPAT_INSTALL_PATH", install_path);
        }
        for key in RESERVED_ENVIRONMENT_KEYS {
            command.env_remove(key);
        }
        command
            .env("GAMEID", &game.game_id)
            .env("PROTONPATH", &game.proton)
            .env("STORE", &game.store)
            .env("WINEPREFIX", prefix)
            .envs(&game.environment);
        if let Some(cwd) = &game.working_directory {
            command.current_dir(cwd);
        }
        Ok(command)
    }
}

fn clean_text(value: &str, field: &str, allow_empty: bool) -> Result<String> {
    if value.contains('\0') || (!allow_empty && value.trim().is_empty()) {
        return Err(BottlesError::InvalidInput(format!("Invalid {field}")));
    }
    Ok(value.to_owned())
}

fn normalize_absolute(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            Component::RootDir => result.push("/"),
            other => result.push(other.as_os_str()),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_repository() -> (Paths, UmuRepository) {
        let mut paths = Paths::detect();
        paths.base = std::env::temp_dir().join(format!("bottles-rs-umu-{}", Uuid::new_v4()));
        paths.umu = paths.base.join("umu");
        (paths.clone(), UmuRepository::new(None, &paths))
    }

    #[test]
    fn managed_prefix_must_stay_inside_root() {
        let (paths, repository) = temp_repository();
        let game = repository.new_game(
            "Game",
            PathBuf::from("/tmp/game.exe"),
            DEFAULT_PROTON_VALUE.to_owned(),
            "umu-default".to_owned(),
            "none".to_owned(),
            false,
            false,
        );
        assert!(repository
            .prefix_path(&game)
            .unwrap()
            .starts_with(&repository.root));
        let mut invalid = game.clone();
        invalid.prefix.path = "../escape".to_owned();
        assert!(invalid.validate().is_ok());
        assert!(invalid.prefix.resolve(&paths.umu).is_err());
    }

    #[test]
    fn repository_saves_loads_and_lists_games() {
        let (paths, repository) = temp_repository();
        let game = repository.new_game(
            "Game",
            PathBuf::from("/tmp/game.exe"),
            DEFAULT_PROTON_VALUE.to_owned(),
            "umu-default".to_owned(),
            "steam".to_owned(),
            false,
            false,
        );
        let id = game.id;
        repository.save(&game).unwrap();
        assert_eq!(repository.load(id).unwrap().name, "Game");
        assert_eq!(repository.list_games().unwrap().len(), 1);
        assert!(repository.delete(id).unwrap());
        fs::remove_dir_all(paths.base).unwrap();
    }
}

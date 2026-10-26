//! Compatibility core for the Rust/egui port of Bottles.
//!
//! The module layout follows the Python implementation.  Generated contracts
//! under `rust/contracts` are embedded into this crate so unit tests can prove
//! that config fields, settings, CLI commands, and UI screens have not drifted
//! from the source tree.

pub mod config;
pub mod error;
pub mod health;
pub mod journal;
pub mod library;
pub mod manager;
pub mod parity;
pub mod paths;
pub mod registry_rules;
pub mod settings;
pub mod state;
pub mod umu;
pub mod wine;

pub use config::{BottleConfig, BottleParams, BottleSandboxParams, ExternalProgram, RegistryRule};
pub use error::{BottlesError, Result};
pub use manager::{Bottle, BottleManager};
pub use paths::{Paths, APP_ID, APP_NAME, APP_VERSION, DOC_URL};

/// CLI commands generated from `bottles/frontend/cli/cli.py`.
pub const CLI_COMMANDS: &[&str] = &[
    "info",
    "list",
    "programs",
    "add",
    "tools",
    "reg",
    "reg-rules",
    "edit",
    "new",
    "run",
    "stop",
    "standalone",
    "shell",
    "umu",
    "autostart",
];

/// Shared application context for both front ends.
#[derive(Debug, Clone)]
pub struct AppContext {
    pub paths: Paths,
    pub manager: BottleManager,
    pub settings: settings::AppSettings,
    pub library: library::Library,
    pub journal: journal::Journal,
    pub user_data: settings::UserData,
    pub umu_root: std::path::PathBuf,
}

impl AppContext {
    /// Open the real Bottles data directory and create required directories.
    pub fn load() -> Result<Self> {
        let paths = Paths::detect();
        paths.ensure_app_dirs()?;
        Self::with_paths(paths)
    }

    /// Open a custom data root; used by tests and the CLI when a caller
    /// overrides `XDG_DATA_HOME`.
    pub fn with_paths(paths: Paths) -> Result<Self> {
        paths.ensure_app_dirs()?;
        let umu_root = paths.clone().umu.clone();
        Ok(Self {
            manager: BottleManager::new(paths.clone()),
            settings: settings::AppSettings::load(),
            library: library::Library::load(&paths)?,
            journal: journal::Journal::load(&paths)?,
            user_data: settings::UserData::load(&paths)?,
            umu_root,
            paths,
        })
    }

    pub fn umu_repository(&self) -> umu::UmuRepository {
        let custom = if self.settings.umu_data_path.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(&self.settings.umu_data_path))
        };
        umu::UmuRepository::new(custom, &self.paths)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.settings = settings::AppSettings::load();
        self.library.reload()?;
        Ok(())
    }
}

pub mod prelude {
    pub use crate::error::{BottlesError, Result};
    pub use crate::paths::{Paths, APP_ID, APP_NAME, APP_VERSION};
    pub use crate::{AppContext, Bottle, BottleConfig, BottleManager};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_command_surface_matches_python_contract() {
        let contract = parity::contracts().cli.commands.clone();
        let actual: Vec<String> = contract
            .iter()
            .map(|command| command.name.clone())
            .collect();
        assert_eq!(actual, CLI_COMMANDS);
    }

    #[test]
    fn embedded_contracts_have_expected_source_counts() {
        let contracts = parity::contracts();
        assert_eq!(contracts.ui.screens.len(), 78);
        assert_eq!(contracts.settings.keys.len(), 33);
        assert!(contracts.modules.modules.len() > 200);
        assert!(parity::inventory().len() > 350);
    }

    #[test]
    fn app_context_can_use_an_isolated_data_root() {
        let mut paths = Paths::detect();
        paths.base =
            std::env::temp_dir().join(format!("bottles-rs-context-{}", uuid::Uuid::new_v4()));
        paths.bottles = paths.base.join("bottles");
        paths.library = paths.base.join("library.yml");
        paths.journal = paths.base.join("journal.yml");
        paths.data = paths.base.join("data.yml");
        paths.temp = paths.base.join("temp");
        paths.runners = paths.base.join("runners");
        paths.umu = paths.base.join("umu");
        let context = AppContext::with_paths(paths.clone()).unwrap();
        assert!(context.paths.base.is_dir());
        assert!(context.paths.library.is_file());
        std::fs::remove_dir_all(paths.base).unwrap();
    }
}

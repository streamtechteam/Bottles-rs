use bottles_core::config::{BottleParams, ExternalProgram, RegistryRule};
use bottles_core::journal::JournalPeriod;
use bottles_core::manager::{NewBottleOptions, WinePrefixCandidate};
use bottles_core::parity::{inventory, summary, ParityStatus};
use bottles_core::registry_rules::RegistryRuleManager;
use bottles_core::umu::{UmuGame, DEFAULT_PROTON_VALUE};
use bottles_core::wine::{split_arguments, WineLauncher, WineTool};
use bottles_core::{AppContext, Bottle, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    Bottles,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    Details,
    Settings,
    Dependencies,
    Installers,
    RegistryRules,
    Snapshots,
    TaskManager,
    Importer,
    Journal,
    Preferences,
    Components,
    Health,
    Parity,
    Umu,
    About,
    Help,
}

impl View {
    pub fn title(self) -> &'static str {
        match self {
            Self::Main => "Bottles",
            Self::Details => "Details",
            Self::Settings => "Settings",
            Self::Dependencies => "Dependencies",
            Self::Installers => "Installers",
            Self::RegistryRules => "Registry Rules",
            Self::Snapshots => "Snapshots",
            Self::TaskManager => "Task Manager",
            Self::Importer => "Import",
            Self::Journal => "Journal",
            Self::Preferences => "Preferences",
            Self::Components => "Components",
            Self::Health => "System Health",
            Self::Parity => "Parity",
            Self::Umu => "UMU",
            Self::About => "About Bottles",
            Self::Help => "Keyboard Shortcuts",
        }
    }

    pub fn is_bottle_subview(self) -> bool {
        matches!(
            self,
            Self::Details
                | Self::Settings
                | Self::Dependencies
                | Self::Installers
                | Self::RegistryRules
                | Self::Snapshots
                | Self::TaskManager
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    Run,
    Shortcut,
    Rename,
    Duplicate,
    Delete,
    Backup,
    LaunchOptions,
    Environment,
    Registry,
    Help,
    About,
}

#[derive(Debug)]
pub struct AppState {
    pub context: AppContext,
    pub main_tab: MainTab,
    pub view: View,
    pub back_stack: Vec<View>,
    pub bottles: Vec<Bottle>,
    pub components: BTreeMap<String, Vec<String>>,
    pub importer_candidates: Vec<WinePrefixCandidate>,
    pub umu_games: Vec<UmuGame>,
    pub selected: Option<String>,
    pub search: String,
    pub search_visible: bool,
    pub installer_query: String,
    pub dependency_query: String,
    pub component_query: String,
    pub parity_search: String,
    pub parity_status: Option<ParityStatus>,
    pub journal_period: JournalPeriod,
    pub toast: Option<String>,
    pub error: Option<String>,
    pub show_add_menu: bool,
    pub show_main_menu: bool,
    pub show_power_menu: bool,
    pub show_context_menu: bool,
    pub create_dialog: bool,
    pub run_dialog: bool,
    pub shortcut_dialog: bool,
    pub rename_dialog: bool,
    pub duplicate_dialog: bool,
    pub delete_dialog: bool,
    pub backup_dialog: bool,
    pub launch_options_dialog: bool,
    pub env_dialog: bool,
    pub registry_dialog: bool,
    pub help_dialog: bool,
    pub about_dialog: bool,
    pub show_hidden_programs: bool,
    pub new_bottle_name: String,
    pub new_bottle_environment: String,
    pub new_bottle_arch: String,
    pub new_bottle_runner: String,
    pub new_bottle_custom_path: String,
    pub new_bottle_sandbox: bool,
    pub rename_input: String,
    pub duplicate_input: String,
    pub run_executable: String,
    pub exec_arguments: String,
    pub exec_terminal: bool,
    pub exec_winebridge: bool,
    pub backup_path: String,
    pub program_name: String,
    pub program_path: String,
    pub program_arguments: String,
    pub edit_runner: String,
    pub edit_windows: String,
    pub edit_arch: String,
    pub edit_sync: String,
    pub edit_d7vk: String,
    pub edit_dxvk_version: String,
    pub edit_nvapi: String,
    pub edit_vkd3d_version: String,
    pub edit_latencyflex: String,
    pub edit_params: BottleParams,
    pub edit_dxvk: bool,
    pub edit_vkd3d: bool,
    pub edit_mangohud: bool,
    pub edit_gamescope: bool,
    pub env_key: String,
    pub env_value: String,
    pub rule_name: String,
    pub rule_description: String,
    pub rule_keys: String,
    pub importer_manual_path: String,
    pub umu_name: String,
    pub umu_executable: String,
    pub umu_proton: String,
}

impl AppState {
    pub fn load() -> Result<Self> {
        let context = AppContext::load()?;
        Ok(Self::with_context(context))
    }

    pub fn with_context(context: AppContext) -> Self {
        let main_tab = if context.settings.startup_view == "page_bottles" {
            MainTab::Bottles
        } else {
            MainTab::Library
        };
        let new_bottle_runner = context.manager.latest_runner();
        let umu_proton = if context.settings.umu_proton.is_empty() {
            DEFAULT_PROTON_VALUE.to_owned()
        } else {
            context.settings.umu_proton.clone()
        };
        let mut state = Self {
            context,
            main_tab,
            view: View::Main,
            back_stack: Vec::new(),
            bottles: Vec::new(),
            components: BTreeMap::new(),
            importer_candidates: Vec::new(),
            umu_games: Vec::new(),
            selected: None,
            search: String::new(),
            search_visible: false,
            installer_query: String::new(),
            dependency_query: String::new(),
            component_query: String::new(),
            parity_search: String::new(),
            parity_status: None,
            journal_period: JournalPeriod::Today,
            toast: None,
            error: None,
            show_add_menu: false,
            show_main_menu: false,
            show_power_menu: false,
            show_context_menu: false,
            create_dialog: false,
            run_dialog: false,
            shortcut_dialog: false,
            rename_dialog: false,
            duplicate_dialog: false,
            delete_dialog: false,
            backup_dialog: false,
            launch_options_dialog: false,
            env_dialog: false,
            registry_dialog: false,
            help_dialog: false,
            about_dialog: false,
            show_hidden_programs: false,
            new_bottle_name: String::new(),
            new_bottle_environment: "application".to_owned(),
            new_bottle_arch: "win64".to_owned(),
            new_bottle_runner,
            new_bottle_custom_path: String::new(),
            new_bottle_sandbox: false,
            rename_input: String::new(),
            duplicate_input: String::new(),
            run_executable: String::new(),
            exec_arguments: String::new(),
            exec_terminal: false,
            exec_winebridge: false,
            backup_path: String::new(),
            program_name: String::new(),
            program_path: String::new(),
            program_arguments: String::new(),
            edit_runner: String::new(),
            edit_windows: String::new(),
            edit_arch: String::new(),
            edit_sync: String::new(),
            edit_d7vk: String::new(),
            edit_dxvk_version: String::new(),
            edit_nvapi: String::new(),
            edit_vkd3d_version: String::new(),
            edit_latencyflex: String::new(),
            edit_params: BottleParams::default(),
            edit_dxvk: false,
            edit_vkd3d: false,
            edit_mangohud: false,
            edit_gamescope: false,
            env_key: String::new(),
            env_value: String::new(),
            rule_name: String::new(),
            rule_description: String::new(),
            rule_keys: String::new(),
            importer_manual_path: String::new(),
            umu_name: String::new(),
            umu_executable: String::new(),
            umu_proton,
        };
        state.refresh_all();
        state
    }

    pub fn refresh_all(&mut self) {
        self.bottles = self.context.manager.list_bottles().unwrap_or_else(|error| {
            self.set_error(error);
            Vec::new()
        });
        self.components = self.context.manager.components();
        self.importer_candidates = self.context.manager.search_wineprefixes(&[]);
        self.umu_games = self
            .context
            .umu_repository()
            .list_games()
            .unwrap_or_else(|error| {
                self.set_error(error);
                Vec::new()
            });
        if self.selected.is_none() {
            if let Some(first) = self.bottles.first() {
                self.select_bottle(first.name().to_owned());
            }
        }
        if let Err(error) = self.context.library.reload() {
            self.set_error(error);
        }
    }

    pub fn set_error(&mut self, error: impl ToString) {
        self.error = Some(error.to_string());
        self.toast = None;
    }

    pub fn set_toast(&mut self, message: impl Into<String>) {
        self.toast = Some(message.into());
        self.error = None;
    }

    pub fn selected_bottle(&self) -> Option<Bottle> {
        let name = self.selected.as_deref()?;
        self.bottles
            .iter()
            .find(|bottle| bottle.name() == name)
            .cloned()
    }

    pub fn select_bottle(&mut self, name: String) {
        let Some(bottle) = self
            .bottles
            .iter()
            .find(|bottle| bottle.name() == name)
            .cloned()
        else {
            return;
        };
        self.selected = Some(name);
        self.rename_input = bottle.config.name.clone();
        self.duplicate_input = format!("{} copy", bottle.config.name);
        self.edit_runner = bottle.config.runner.clone();
        self.edit_windows = bottle.config.windows.clone();
        self.edit_arch = bottle.config.arch.clone();
        self.edit_sync = bottle.config.parameters.sync.clone();
        self.edit_d7vk = bottle.config.d7vk.clone();
        self.edit_dxvk_version = bottle.config.dxvk.clone();
        self.edit_nvapi = bottle.config.nvapi.clone();
        self.edit_vkd3d_version = bottle.config.vkd3d.clone();
        self.edit_latencyflex = bottle.config.latency_flex.clone();
        self.edit_params = bottle.config.parameters.clone();
        self.edit_dxvk = bottle.config.parameters.dxvk;
        self.edit_vkd3d = bottle.config.parameters.vkd3d;
        self.edit_mangohud = bottle.config.parameters.mangohud;
        self.edit_gamescope = bottle.config.parameters.gamescope;
    }

    pub fn dialog_open(&self, dialog: Dialog) -> bool {
        match dialog {
            Dialog::Run => self.run_dialog,
            Dialog::Shortcut => self.shortcut_dialog,
            Dialog::Rename => self.rename_dialog,
            Dialog::Duplicate => self.duplicate_dialog,
            Dialog::Delete => self.delete_dialog,
            Dialog::Backup => self.backup_dialog,
            Dialog::LaunchOptions => self.launch_options_dialog,
            Dialog::Environment => self.env_dialog,
            Dialog::Registry => self.registry_dialog,
            Dialog::Help => self.help_dialog,
            Dialog::About => self.about_dialog,
        }
    }

    pub fn close_dialog(&mut self, dialog: Dialog) {
        match dialog {
            Dialog::Run => self.run_dialog = false,
            Dialog::Shortcut => self.shortcut_dialog = false,
            Dialog::Rename => self.rename_dialog = false,
            Dialog::Duplicate => self.duplicate_dialog = false,
            Dialog::Delete => self.delete_dialog = false,
            Dialog::Backup => self.backup_dialog = false,
            Dialog::LaunchOptions => self.launch_options_dialog = false,
            Dialog::Environment => self.env_dialog = false,
            Dialog::Registry => self.registry_dialog = false,
            Dialog::Help => self.help_dialog = false,
            Dialog::About => self.about_dialog = false,
        }
    }

    pub fn navigate(&mut self, view: View) {
        if self.view != view {
            self.back_stack.push(self.view);
            self.view = view;
        }
        self.close_menus();
    }

    pub fn back(&mut self) {
        self.view = self.back_stack.pop().unwrap_or(View::Main);
        self.close_menus();
    }

    pub fn go_main(&mut self) {
        self.view = View::Main;
        self.back_stack.clear();
        self.close_menus();
    }

    pub fn open_bottle(&mut self, name: String) {
        self.select_bottle(name);
        self.navigate(View::Details);
    }

    pub fn close_menus(&mut self) {
        self.show_add_menu = false;
        self.show_main_menu = false;
        self.show_power_menu = false;
        self.show_context_menu = false;
    }

    fn launcher(&self) -> Option<(Bottle, WineLauncher)> {
        let bottle = self.selected_bottle()?;
        let launcher = WineLauncher::new(
            self.context.paths.clone(),
            bottle.config.clone(),
            bottle.prefix.clone(),
        );
        Some((bottle, launcher))
    }

    pub fn create_bottle(&mut self) {
        let path = if self.new_bottle_custom_path.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.new_bottle_custom_path.trim()))
        };
        let options = NewBottleOptions {
            name: self.new_bottle_name.clone(),
            environment: self.new_bottle_environment.clone(),
            arch: self.new_bottle_arch.clone(),
            runner: if self.new_bottle_runner.trim().is_empty() {
                None
            } else {
                Some(self.new_bottle_runner.clone())
            },
            path,
            sandbox: self.new_bottle_sandbox,
            ..NewBottleOptions::default()
        };
        match self.context.manager.create_bottle(options) {
            Ok(bottle) => {
                self.new_bottle_name.clear();
                self.new_bottle_custom_path.clear();
                self.create_dialog = false;
                self.set_toast(format!("Created {}", bottle.config.name));
                self.refresh_all();
                self.select_bottle(bottle.config.name);
                self.main_tab = MainTab::Bottles;
                self.go_main();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn rename_selected(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let new_name = self.rename_input.clone();
        match self.context.manager.rename_bottle(bottle.name(), &new_name) {
            Ok(_) => {
                let old_name = bottle.name().to_owned();
                if let Err(error) = self.context.library.rename_bottle(&old_name, &new_name) {
                    self.set_error(error);
                } else {
                    self.set_toast("Bottle renamed");
                }
                self.rename_dialog = false;
                self.refresh_all();
                self.select_bottle(new_name);
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn duplicate_selected(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let new_name = self.duplicate_input.clone();
        match self
            .context
            .manager
            .duplicate_bottle(bottle.name(), &new_name)
        {
            Ok(bottle) => {
                self.duplicate_dialog = false;
                self.set_toast("Bottle duplicated");
                self.refresh_all();
                self.select_bottle(bottle.config.name);
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn delete_selected(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        match self.context.manager.delete_bottle(bottle.name()) {
            Ok(()) => {
                self.selected = None;
                self.delete_dialog = false;
                self.set_toast("Bottle deleted");
                self.refresh_all();
                self.go_main();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn launch_selected_executable(&mut self) {
        let Some((_bottle, launcher)) = self.launcher() else {
            return;
        };
        let executable = self.run_executable.trim().to_owned();
        if executable.is_empty() {
            self.set_error("Enter an executable path");
            return;
        }
        let arguments = split_arguments(&self.exec_arguments);
        match launcher
            .plan(&executable, &arguments, None)
            .and_then(|plan| launcher.spawn(&plan))
        {
            Ok(_) => {
                self.set_toast(format!("Launched {executable}"));
                self.run_dialog = false;
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn launch_program(&mut self, program: &ExternalProgram) {
        let Some((_bottle, launcher)) = self.launcher() else {
            return;
        };
        let extra = split_arguments(&self.exec_arguments);
        match launcher
            .program_plan(program, &extra)
            .and_then(|plan| launcher.spawn(&plan))
        {
            Ok(_) => self.set_toast(format!("Launched {}", program.display_name())),
            Err(error) => self.set_error(error),
        }
    }

    pub fn add_selected_program(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let name = self.program_name.trim().to_owned();
        let path = self.program_path.trim().to_owned();
        let arguments = self.program_arguments.clone();
        match self.context.manager.add_program(
            bottle.name(),
            &name,
            &path,
            Some(&arguments),
            false,
            false,
            false,
            false,
        ) {
            Ok(_) => {
                self.program_name.clear();
                self.program_path.clear();
                self.program_arguments.clear();
                self.shortcut_dialog = false;
                self.set_toast("Program added");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn save_selected_configuration(&mut self) {
        let Some(mut bottle) = self.selected_bottle() else {
            return;
        };
        bottle.config.runner = self.edit_runner.clone();
        bottle.config.windows = self.edit_windows.clone();
        bottle.config.arch = self.edit_arch.clone();
        bottle.config.d7vk = self.edit_d7vk.clone();
        bottle.config.dxvk = self.edit_dxvk_version.clone();
        bottle.config.nvapi = self.edit_nvapi.clone();
        bottle.config.vkd3d = self.edit_vkd3d_version.clone();
        bottle.config.latency_flex = self.edit_latencyflex.clone();
        bottle.config.parameters = self.edit_params.clone();
        bottle.config.parameters.sync = self.edit_sync.clone();
        bottle.config.parameters.dxvk = self.edit_dxvk;
        bottle.config.parameters.vkd3d = self.edit_vkd3d;
        bottle.config.parameters.mangohud = self.edit_mangohud;
        bottle.config.parameters.gamescope = self.edit_gamescope;
        match self.context.manager.save_config(&bottle) {
            Ok(()) => {
                self.set_toast("Configuration saved");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn add_environment_variable(&mut self) {
        let Some(mut bottle) = self.selected_bottle() else {
            return;
        };
        let key = self.env_key.trim().to_owned();
        let value = self.env_value.clone();
        if key.is_empty() {
            self.set_error("Environment variable name is empty");
            return;
        }
        match bottle
            .config
            .set_environment_variable(&key, &value)
            .and_then(|_| self.context.manager.save_config(&bottle))
        {
            Ok(()) => {
                self.env_key.clear();
                self.env_value.clear();
                self.env_dialog = false;
                self.set_toast("Environment variable saved");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn add_registry_rule(&mut self) {
        let Some(mut bottle) = self.selected_bottle() else {
            return;
        };
        let rule = RegistryRule {
            name: self.rule_name.trim().to_owned(),
            description: self.rule_description.clone(),
            keys: self.rule_keys.clone(),
            triggers: vec!["components".to_owned()],
            run_once: false,
        };
        if rule.name.is_empty() || rule.keys.trim().is_empty() {
            self.set_error("A rule name and registry keys are required");
            return;
        }
        RegistryRuleManager::upsert_rule(&mut bottle.config, rule);
        match self.context.manager.save_config(&bottle) {
            Ok(()) => {
                self.rule_name.clear();
                self.rule_description.clear();
                self.rule_keys.clear();
                self.registry_dialog = false;
                self.set_toast("Registry rule saved");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn apply_registry_rules(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        match RegistryRuleManager::apply_rules(
            &self.context.paths,
            &bottle.config,
            &bottle.prefix,
            None,
            None,
        ) {
            Ok(applied) => self.set_toast(format!("Applied {} registry rule(s)", applied.len())),
            Err(error) => self.set_error(error),
        }
    }

    pub fn full_backup(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let destination = if self.backup_path.trim().is_empty() {
            self.context
                .paths
                .data
                .join(format!("{}.tar.gz", bottle.name()))
        } else {
            PathBuf::from(self.backup_path.trim())
        };
        match self
            .context
            .manager
            .create_backup(bottle.name(), &destination)
        {
            Ok(path) => {
                self.backup_dialog = false;
                self.set_toast(format!("Backup saved to {}", path.display()));
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn export_configuration(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let destination = if self.backup_path.trim().is_empty() {
            self.context
                .paths
                .data
                .join(format!("{}.yml", bottle.name()))
        } else {
            PathBuf::from(self.backup_path.trim())
        };
        let result = bottle.config.to_yaml_string().and_then(|content| {
            fs::write(&destination, content)
                .map_err(|error| bottles_core::error::io_error(&destination, error.to_string()))
        });
        match result {
            Ok(()) => {
                self.backup_dialog = false;
                self.set_toast(format!(
                    "Configuration exported to {}",
                    destination.display()
                ));
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn repair_selected(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        match self.context.manager.repair_bottle(bottle.name()) {
            Ok(()) => self.set_toast("Bottle repair started"),
            Err(error) => self.set_error(error),
        }
    }

    pub fn run_wine_tool(&mut self, tool: WineTool, arguments: &[&str]) {
        let Some((_bottle, launcher)) = self.launcher() else {
            return;
        };
        let arguments: Vec<String> = arguments.iter().map(|value| (*value).to_owned()).collect();
        match launcher
            .tool_plan(tool, &arguments)
            .and_then(|plan| launcher.spawn(&plan))
        {
            Ok(_) => self.set_toast(format!("Started {}", tool.name())),
            Err(error) => self.set_error(error),
        }
    }

    pub fn import_candidate(&mut self, index: usize) {
        let Some(candidate) = self.importer_candidates.get(index).cloned() else {
            return;
        };
        match self.context.manager.import_wineprefix(&candidate) {
            Ok(bottle) => {
                self.set_toast(format!("Imported {}", bottle.config.name));
                self.refresh_all();
                self.main_tab = MainTab::Bottles;
                self.go_main();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn add_umu_game(&mut self) {
        let name = self.umu_name.trim().to_owned();
        let executable = self.umu_executable.trim().to_owned();
        if name.is_empty() || executable.is_empty() {
            self.set_error("UMU game name and executable are required");
            return;
        }
        let repository = self.context.umu_repository();
        let game = repository.new_game(
            &name,
            PathBuf::from(executable),
            self.umu_proton.clone(),
            "umu-default".to_owned(),
            "none".to_owned(),
            false,
            false,
        );
        match repository.save(&game) {
            Ok(_) => {
                self.umu_name.clear();
                self.umu_executable.clear();
                self.set_toast("UMU game added");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub fn remove_library_entry(&mut self, id: &str) {
        match self.context.library.remove(id) {
            Ok(true) => self.set_toast("Removed from Library"),
            Ok(false) => self.set_error("Library entry not found"),
            Err(error) => self.set_error(error),
        }
    }

    pub fn save_preferences(&mut self) {
        match self.context.settings.save() {
            Ok(()) => self.set_toast("Preferences saved"),
            Err(error) => self.set_error(format!("GSettings unavailable: {error}")),
        }
    }

    pub fn parity_counts(&self) -> (usize, usize, usize, usize) {
        let items = inventory();
        let ported = items
            .iter()
            .filter(|item| item.status == ParityStatus::Ported)
            .count();
        let partial = items
            .iter()
            .filter(|item| item.status == ParityStatus::Partial)
            .count();
        let inventory_only = items
            .iter()
            .filter(|item| item.status == ParityStatus::InventoryOnly)
            .count();
        let reference_only = items
            .iter()
            .filter(|item| item.status == ParityStatus::ReferenceOnly)
            .count();
        let _ = summary();
        (ported, partial, inventory_only, reference_only)
    }
}

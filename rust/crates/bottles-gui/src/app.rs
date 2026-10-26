use bottles_core::config::RegistryRule;
use bottles_core::health::HealthChecker;
use bottles_core::journal::JournalPeriod;
use bottles_core::manager::{NewBottleOptions, WinePrefixCandidate};
use bottles_core::parity::{inventory, summary, ParityStatus};
use bottles_core::registry_rules::RegistryRuleManager;
use bottles_core::umu::{UmuGame, DEFAULT_PROTON_VALUE};
use bottles_core::wine::WineLauncher;
use bottles_core::{AppContext, Bottle, Result, APP_ID, APP_NAME, APP_VERSION, DOC_URL};
use eframe::egui::{self, RichText};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Library,
    Bottles,
    Details,
    Components,
    Importer,
    Journal,
    Health,
    Preferences,
    Parity,
    Umu,
    About,
}

impl Screen {
    pub const ALL: &'static [(Self, &'static str, &'static str)] = &[
        (Self::Library, "📚", "Library"),
        (Self::Bottles, "🍾", "Bottles"),
        (Self::Details, "⚙", "Details"),
        (Self::Components, "🧩", "Components"),
        (Self::Importer, "📥", "Importer"),
        (Self::Journal, "📝", "Journal"),
        (Self::Health, "🩺", "Health"),
        (Self::Preferences, "🎛", "Preferences"),
        (Self::Parity, "📋", "Parity"),
        (Self::Umu, "🎮", "UMU"),
        (Self::About, "ℹ", "About"),
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Bottle,
    Preferences,
    Dependencies,
    Programs,
    RegistryRules,
    TaskManager,
    Versioning,
}

impl DetailTab {
    pub const ALL: &'static [Self] = &[
        Self::Bottle,
        Self::Preferences,
        Self::Dependencies,
        Self::Programs,
        Self::RegistryRules,
        Self::TaskManager,
        Self::Versioning,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Bottle => "Bottle",
            Self::Preferences => "Preferences",
            Self::Dependencies => "Dependencies",
            Self::Programs => "Programs",
            Self::RegistryRules => "Registry rules",
            Self::TaskManager => "Task manager",
            Self::Versioning => "Versioning",
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub context: AppContext,
    pub screen: Screen,
    pub detail_tab: DetailTab,
    pub bottles: Vec<Bottle>,
    pub components: BTreeMap<String, Vec<String>>,
    pub importer_candidates: Vec<WinePrefixCandidate>,
    pub umu_games: Vec<UmuGame>,
    pub selected: Option<String>,
    pub search: String,
    pub parity_search: String,
    pub parity_status: Option<ParityStatus>,
    pub journal_period: JournalPeriod,
    pub toast: Option<String>,
    pub error: Option<String>,
    pub confirm_delete: bool,
    pub new_bottle_name: String,
    pub new_bottle_environment: String,
    pub new_bottle_arch: String,
    pub new_bottle_custom_path: String,
    pub rename_input: String,
    pub duplicate_input: String,
    pub run_executable: String,
    pub program_name: String,
    pub program_path: String,
    pub program_arguments: String,
    pub edit_runner: String,
    pub edit_windows: String,
    pub edit_arch: String,
    pub edit_sync: String,
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
        let screen = if context.settings.startup_view == "page_bottles" {
            Screen::Bottles
        } else {
            Screen::Library
        };
        let mut state = Self {
            context,
            screen,
            detail_tab: DetailTab::Bottle,
            bottles: Vec::new(),
            components: BTreeMap::new(),
            importer_candidates: Vec::new(),
            umu_games: Vec::new(),
            selected: None,
            search: String::new(),
            parity_search: String::new(),
            parity_status: None,
            journal_period: JournalPeriod::Today,
            toast: None,
            error: None,
            confirm_delete: false,
            new_bottle_name: String::new(),
            new_bottle_environment: "gaming".to_owned(),
            new_bottle_arch: "win64".to_owned(),
            new_bottle_custom_path: String::new(),
            rename_input: String::new(),
            duplicate_input: String::new(),
            run_executable: String::new(),
            program_name: String::new(),
            program_path: String::new(),
            program_arguments: String::new(),
            edit_runner: String::new(),
            edit_windows: String::new(),
            edit_arch: String::new(),
            edit_sync: String::new(),
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
            umu_proton: DEFAULT_PROTON_VALUE.to_owned(),
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
        let _ = self.context.refresh();
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
        self.edit_dxvk = bottle.config.parameters.dxvk;
        self.edit_vkd3d = bottle.config.parameters.vkd3d;
        self.edit_mangohud = bottle.config.parameters.mangohud;
        self.edit_gamescope = bottle.config.parameters.gamescope;
    }

    fn create_bottle(&mut self) {
        let path = if self.new_bottle_custom_path.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.new_bottle_custom_path.trim()))
        };
        let options = NewBottleOptions {
            name: self.new_bottle_name.clone(),
            environment: self.new_bottle_environment.clone(),
            arch: self.new_bottle_arch.clone(),
            path,
            ..NewBottleOptions::default()
        };
        match self.context.manager.create_bottle(options) {
            Ok(bottle) => {
                self.new_bottle_name.clear();
                self.set_toast(format!("Created {}", bottle.config.name));
                self.refresh_all();
                self.select_bottle(bottle.config.name);
            }
            Err(error) => self.set_error(error),
        }
    }

    fn rename_selected(&mut self) {
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
                self.refresh_all();
                self.select_bottle(new_name);
            }
            Err(error) => self.set_error(error),
        }
    }

    fn duplicate_selected(&mut self) {
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
                self.set_toast("Bottle duplicated");
                self.refresh_all();
                self.select_bottle(bottle.config.name);
            }
            Err(error) => self.set_error(error),
        }
    }

    fn delete_selected(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        match self.context.manager.delete_bottle(bottle.name()) {
            Ok(()) => {
                self.selected = None;
                self.set_toast("Bottle deleted");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn launch_selected_executable(&mut self) {
        let Some(bottle) = self.selected_bottle() else {
            return;
        };
        let executable = self.run_executable.trim().to_owned();
        if executable.is_empty() {
            self.set_error("Enter an executable path");
            return;
        }
        let launcher = WineLauncher::new(
            self.context.paths.clone(),
            bottle.config.clone(),
            bottle.prefix.clone(),
        );
        match launcher
            .plan(&executable, &[], None)
            .and_then(|plan| launcher.spawn(&plan))
        {
            Ok(_) => self.set_toast(format!("Launched {executable}")),
            Err(error) => self.set_error(error),
        }
    }

    fn add_selected_program(&mut self) {
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
                self.set_toast("Program added");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn save_selected_configuration(&mut self) {
        let Some(mut bottle) = self.selected_bottle() else {
            return;
        };
        bottle.config.runner = self.edit_runner.clone();
        bottle.config.windows = self.edit_windows.clone();
        bottle.config.arch = self.edit_arch.clone();
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

    fn add_environment_variable(&mut self) {
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
                self.set_toast("Environment variable saved");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn add_registry_rule(&mut self) {
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
                self.set_toast("Registry rule saved");
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn apply_registry_rules(&mut self) {
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

    fn import_candidate(&mut self, index: usize) {
        let Some(candidate) = self.importer_candidates.get(index).cloned() else {
            return;
        };
        match self.context.manager.import_wineprefix(&candidate) {
            Ok(bottle) => {
                self.set_toast(format!("Imported {}", bottle.config.name));
                self.refresh_all();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn add_umu_game(&mut self) {
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

    fn save_preferences(&mut self) {
        match self.context.settings.save() {
            Ok(()) => self.set_toast("Preferences saved"),
            Err(error) => self.set_error(format!("GSettings unavailable: {error}")),
        }
    }
}

pub struct BottlesApp {
    pub state: AppState,
}

impl BottlesApp {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    fn configure(&self, ctx: &egui::Context) {
        ctx.set_visuals(if self.state.context.settings.dark_theme {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        ctx.set_pixels_per_point(self.state.context.settings.font_scale as f32);
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading(APP_NAME);
        ui.label(RichText::new(format!("v{APP_VERSION} — Rust/egui")).weak());
        ui.separator();
        for (screen, icon, label) in Screen::ALL {
            if ui
                .selectable_label(self.state.screen == *screen, format!("{icon}  {label}"))
                .clicked()
            {
                self.state.screen = *screen;
            }
        }
        ui.separator();
        if ui.button("Refresh").clicked() {
            self.state.refresh_all();
            self.state.set_toast("Refreshed");
        }
        ui.label(RichText::new(format!("{} bottles", self.state.bottles.len())).weak());
        ui.label(RichText::new(format!("{} source-derived features", inventory().len())).weak());
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(
                egui::TextEdit::singleline(&mut self.state.search)
                    .hint_text("Filter bottles, library, and parity inventory"),
            );
            if ui.button("Clear").clicked() {
                self.state.search.clear();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(APP_ID).monospace().weak());
            });
        });
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.state.error {
            ui.colored_label(egui::Color32::RED, format!("Error: {error}"));
        } else if let Some(toast) = &self.state.toast {
            ui.colored_label(egui::Color32::DARK_GREEN, toast);
        } else {
            ui.label("The Python implementation remains the behavioral reference.");
        }
    }

    fn library_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Library");
        let entries = self.state.context.library.entries.clone();
        let query = self.state.search.to_ascii_lowercase();
        let mut visible = 0;
        egui::ScrollArea::vertical().show(ui, |ui| {
            if entries.is_empty() {
                ui.label("No library entries. Add a program to a bottle, then use the Python app or a future ported view to add it to the library.");
            }
            for (id, entry) in entries {
                let name = entry
                    .as_mapping()
                    .and_then(|map| map.get(&serde_yaml::Value::String("name".to_owned())))
                    .and_then(|value| value.as_str())
                    .unwrap_or("Untitled")
                    .to_owned();
                if !query.is_empty() && !name.to_ascii_lowercase().contains(&query) {
                    continue;
                }
                visible += 1;
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Remove").clicked()
                                && self.state.context.library.remove(&id).is_ok()
                            {
                                self.state.set_toast(format!("Removed {name}"));
                            }
                        });
                    });
                    ui.label(RichText::new(id).monospace().weak());
                });
            }
            if visible == 0 && !query.is_empty() {
                ui.label("No library entries match the search.");
            }
        });
    }

    fn bottles_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bottles");
        ui.columns(2, |columns| {
            let ui = &mut columns[0];
            ui.group(|ui| {
                ui.heading("Create bottle");
                labeled_text(ui, "Name", &mut self.state.new_bottle_name);
                ui.horizontal(|ui| {
                    ui.label("Environment");
                    egui::ComboBox::from_id_source("new-environment")
                        .selected_text(&self.state.new_bottle_environment)
                        .show_ui(ui, |ui| {
                            for value in ["gaming", "application", "custom"] {
                                ui.selectable_value(
                                    &mut self.state.new_bottle_environment,
                                    value.to_owned(),
                                    value,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Architecture");
                    egui::ComboBox::from_id_source("new-arch")
                        .selected_text(&self.state.new_bottle_arch)
                        .show_ui(ui, |ui| {
                            for value in ["win64", "win32"] {
                                ui.selectable_value(
                                    &mut self.state.new_bottle_arch,
                                    value.to_owned(),
                                    value,
                                );
                            }
                        });
                });
                labeled_text(
                    ui,
                    "Custom parent path",
                    &mut self.state.new_bottle_custom_path,
                );
                if ui.button("Create").clicked() {
                    self.state.create_bottle();
                }
            });
            let ui = &mut columns[1];
            ui.group(|ui| {
                ui.heading("Bottle actions");
                if let Some(bottle) = self.state.selected_bottle() {
                    ui.label(RichText::new(bottle.name()).strong());
                    labeled_text(ui, "Rename", &mut self.state.rename_input);
                    if ui.button("Rename").clicked() {
                        self.state.rename_selected();
                    }
                    labeled_text(ui, "Duplicate as", &mut self.state.duplicate_input);
                    if ui.button("Duplicate").clicked() {
                        self.state.duplicate_selected();
                    }
                    labeled_text(ui, "Run executable", &mut self.state.run_executable);
                    if ui.button("Run").clicked() {
                        self.state.launch_selected_executable();
                    }
                    ui.separator();
                    if ui.button("Delete bottle").clicked() {
                        self.state.confirm_delete = true;
                    }
                } else {
                    ui.label("Select a bottle to enable actions.");
                }
            });
        });
        ui.separator();
        ui.heading("Installed bottles");
        let query = self.state.search.to_ascii_lowercase();
        let bottles = self.state.bottles.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for bottle in bottles {
                if !query.is_empty() && !bottle.name().to_ascii_lowercase().contains(&query) {
                    continue;
                }
                let selected = self.state.selected.as_deref() == Some(bottle.name());
                if ui.selectable_label(selected, bottle.name()).clicked() {
                    self.state.select_bottle(bottle.name().to_owned());
                }
            }
        });
    }

    fn details_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bottle details");
        let Some(bottle) = self.state.selected_bottle() else {
            ui.label("Create or select a bottle first.");
            return;
        };
        ui.horizontal_wrapped(|ui| {
            for tab in DetailTab::ALL {
                ui.selectable_value(&mut self.state.detail_tab, *tab, tab.label());
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| match self.state.detail_tab {
            DetailTab::Bottle => self.details_bottle_tab(ui, &bottle),
            DetailTab::Preferences => self.details_preferences_tab(ui),
            DetailTab::Dependencies => self.details_dependencies_tab(ui, &bottle),
            DetailTab::Programs => self.details_programs_tab(ui, &bottle),
            DetailTab::RegistryRules => self.details_registry_rules_tab(ui, &bottle),
            DetailTab::TaskManager => self.details_task_manager_tab(ui, &bottle),
            DetailTab::Versioning => self.details_versioning_tab(ui, &bottle),
        });
    }

    fn details_bottle_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.label(RichText::new(&bottle.config.name).heading());
        ui.monospace(bottle.prefix.display().to_string());
        ui.separator();
        egui::Grid::new("bottle-metadata")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Environment");
                ui.label(&bottle.config.environment);
                ui.end_row();
                ui.label("Runner");
                ui.label(&bottle.config.runner);
                ui.end_row();
                ui.label("Windows");
                ui.label(&bottle.config.windows);
                ui.end_row();
                ui.label("Architecture");
                ui.label(&bottle.config.arch);
                ui.end_row();
                ui.label("Created");
                ui.label(&bottle.config.creation_date);
                ui.end_row();
                ui.label("Updated");
                ui.label(&bottle.config.update_date);
                ui.end_row();
            });
    }

    fn details_preferences_tab(&mut self, ui: &mut egui::Ui) {
        labeled_text(ui, "Runner", &mut self.state.edit_runner);
        labeled_text(ui, "Windows version", &mut self.state.edit_windows);
        labeled_text(ui, "Architecture", &mut self.state.edit_arch);
        labeled_text(ui, "Sync", &mut self.state.edit_sync);
        ui.checkbox(&mut self.state.edit_dxvk, "DXVK");
        ui.checkbox(&mut self.state.edit_vkd3d, "VKD3D");
        ui.checkbox(&mut self.state.edit_mangohud, "MangoHud");
        ui.checkbox(&mut self.state.edit_gamescope, "Gamescope");
        if ui.button("Save configuration").clicked() {
            self.state.save_selected_configuration();
        }
        ui.separator();
        ui.heading("Environment variables");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.env_key).hint_text("KEY"));
            ui.add(egui::TextEdit::singleline(&mut self.state.env_value).hint_text("VALUE"));
            if ui.button("Add").clicked() {
                self.state.add_environment_variable();
            }
        });
    }

    fn details_dependencies_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.heading("Installed dependencies");
        if bottle.config.installed_dependencies.is_empty() {
            ui.label("No dependencies recorded in bottle.yml.");
        }
        for dependency in &bottle.config.installed_dependencies {
            ui.label(format!("• {dependency}"));
        }
        ui.separator();
        ui.label(RichText::new("Dependency download/install recipes are still reference-only; see the Parity screen.").weak());
    }

    fn details_programs_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.heading("Programs");
        for program in bottle.config.programs() {
            ui.group(|ui| {
                ui.label(RichText::new(program.display_name()).strong());
                if let Some(path) = &program.path {
                    ui.monospace(path);
                }
            });
        }
        ui.separator();
        labeled_text(ui, "Name", &mut self.state.program_name);
        labeled_text(ui, "Path", &mut self.state.program_path);
        labeled_text(ui, "Arguments", &mut self.state.program_arguments);
        if ui.button("Add program").clicked() {
            self.state.add_selected_program();
        }
    }

    fn details_registry_rules_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.heading("Registry rules");
        for rule in &bottle.config.registry_rules {
            ui.group(|ui| {
                ui.label(RichText::new(&rule.name).strong());
                ui.label(&rule.description);
                ui.monospace(&rule.keys);
            });
        }
        ui.separator();
        labeled_text(ui, "Rule name", &mut self.state.rule_name);
        labeled_text(ui, "Description", &mut self.state.rule_description);
        ui.label("Registry keys");
        ui.add(
            egui::TextEdit::multiline(&mut self.state.rule_keys)
                .desired_rows(8)
                .code_editor(),
        );
        ui.horizontal(|ui| {
            if ui.button("Save rule").clicked() {
                self.state.add_registry_rule();
            }
            if ui.button("Apply rules").clicked() {
                self.state.apply_registry_rules();
            }
        });
    }

    fn details_task_manager_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.heading("Task manager");
        ui.label(format!("Bottle state index: {}", bottle.config.state));
        ui.label("The Python process monitor and playtime tracker are partially ported; use `bottles-cli stop` to stop all Wine processes in this bottle.");
    }

    fn details_versioning_tab(&mut self, ui: &mut egui::Ui, bottle: &Bottle) {
        ui.heading("Versioning");
        ui.label(format!("Versioning enabled: {}", bottle.config.versioning));
        ui.label(format!(
            "Automatic: {}",
            bottle.config.parameters.versioning_automatic
        ));
        ui.label(format!(
            "Compression: {}",
            bottle.config.parameters.versioning_compression
        ));
        ui.label("FVS incremental commits remain reference-only in the Python implementation.");
    }

    fn components_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Components");
        let query = self.state.search.to_ascii_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (kind, values) in &self.state.components {
                ui.collapsing(kind, |ui| {
                    if values.is_empty() {
                        ui.label("None installed");
                    }
                    for value in values {
                        if query.is_empty() || value.to_ascii_lowercase().contains(&query) {
                            ui.label(format!("• {value}"));
                        }
                    }
                });
            }
        });
    }

    fn importer_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Import Wine prefixes");
        labeled_text(ui, "Manual directory", &mut self.state.importer_manual_path);
        if ui.button("Scan manual directory").clicked() {
            let path = PathBuf::from(&self.state.importer_manual_path);
            self.state.importer_candidates =
                self.state.context.manager.search_wineprefixes(&[path]);
        }
        ui.separator();
        let candidates = self.state.importer_candidates.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, candidate) in candidates.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&candidate.name).strong());
                        ui.label(&candidate.manager);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Import").clicked() {
                                self.state.import_candidate(index);
                            }
                        });
                    });
                    ui.monospace(candidate.path.display().to_string());
                    if candidate.lock {
                        ui.label("This source has already been imported.");
                    }
                });
            }
        });
    }

    fn journal_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Journal");
        ui.horizontal(|ui| {
            ui.label("Period");
            egui::ComboBox::from_id_source("journal-period")
                .selected_text(period_label(self.state.journal_period))
                .show_ui(ui, |ui| {
                    for (label, period) in [
                        ("all", JournalPeriod::All),
                        ("today", JournalPeriod::Today),
                        ("yesterday", JournalPeriod::Yesterday),
                        ("week", JournalPeriod::Week),
                        ("month", JournalPeriod::Month),
                    ] {
                        ui.selectable_value(&mut self.state.journal_period, period, label);
                    }
                });
        });
        let entries = self
            .state
            .context
            .journal
            .get(self.state.journal_period)
            .unwrap_or_default();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (id, entry) in entries {
                let mapping = entry.as_mapping();
                let get = |key: &str| {
                    mapping
                        .and_then(|map| map.get(&serde_yaml::Value::String(key.to_owned())))
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_owned()
                };
                ui.group(|ui| {
                    ui.label(format!("{} — {}", get("timestamp"), get("severity")));
                    ui.label(get("message"));
                    ui.label(RichText::new(id).monospace().weak());
                });
            }
        });
    }

    fn health_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Health check");
        let report = HealthChecker::collect().to_json();
        let mut text = serde_json::to_string_pretty(&report).unwrap_or_default();
        ui.add_enabled(
            false,
            egui::TextEdit::multiline(&mut text)
                .desired_rows(20)
                .code_editor(),
        );
    }

    fn preferences_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preferences");
        ui.checkbox(&mut self.state.context.settings.dark_theme, "Dark theme");
        ui.checkbox(
            &mut self.state.context.settings.force_offline,
            "Force offline",
        );
        ui.checkbox(
            &mut self.state.context.settings.notifications,
            "Notifications",
        );
        ui.checkbox(&mut self.state.context.settings.experiments, "Experiments");
        ui.checkbox(
            &mut self.state.context.settings.release_candidate,
            "Release candidate runners",
        );
        ui.checkbox(
            &mut self.state.context.settings.playtime_enabled,
            "Playtime tracking",
        );
        ui.checkbox(
            &mut self.state.context.settings.show_funding,
            "Show funding dialog",
        );
        ui.add(
            egui::Slider::new(&mut self.state.context.settings.font_scale, 1.0..=2.0)
                .text("Font scale"),
        );
        labeled_text(
            ui,
            "UI language",
            &mut self.state.context.settings.ui_language,
        );
        labeled_text(
            ui,
            "Startup view",
            &mut self.state.context.settings.startup_view,
        );
        if ui.button("Save preferences").clicked() {
            self.state.save_preferences();
        }
        ui.separator();
        ui.label("Settings are stored in the same com.usebottles.bottles GSettings schema as the Python app.");
    }

    fn parity_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Source-derived parity inventory");
        let summary = summary();
        ui.horizontal_wrapped(|ui| {
            for (status, count) in summary {
                ui.label(format!("{status}: {count}"));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.add(
                egui::TextEdit::singleline(&mut self.state.parity_search)
                    .hint_text("status, source, or feature"),
            );
            let selected = self
                .state
                .parity_status
                .map(|status| status.label())
                .unwrap_or("all statuses")
                .to_owned();
            egui::ComboBox::from_id_source("parity-status")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.state.parity_status, None, "all statuses");
                    for status in ParityStatus::ALL {
                        ui.selectable_value(
                            &mut self.state.parity_status,
                            Some(*status),
                            status.label(),
                        );
                    }
                });
        });
        let query = self.state.parity_search.trim().to_ascii_lowercase();
        let filter = self.state.parity_status;
        let records = inventory();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for record in records {
                if let Some(filter) = filter {
                    if record.status != filter {
                        continue;
                    }
                }
                let search = if query.is_empty() {
                    String::new()
                } else {
                    format!(
                        "{} {} {} {}",
                        record.kind,
                        record.id,
                        record.source,
                        record.status.label()
                    )
                };
                if !query.is_empty() && !search.to_ascii_lowercase().contains(&query) {
                    continue;
                }
                ui.horizontal(|ui| {
                    status_badge(ui, record.status);
                    ui.label(RichText::new(&record.id).strong());
                    ui.label(RichText::new(&record.kind).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !record.rust_surface.is_empty() {
                            ui.label(RichText::new(&record.rust_surface).monospace().weak());
                        }
                    });
                });
            }
        });
    }

    fn umu_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("UMU games");
        ui.group(|ui| {
            labeled_text(ui, "Name", &mut self.state.umu_name);
            labeled_text(ui, "Executable", &mut self.state.umu_executable);
            labeled_text(ui, "Proton", &mut self.state.umu_proton);
            if ui.button("Add game").clicked() {
                self.state.add_umu_game();
            }
        });
        ui.separator();
        let games = self.state.umu_games.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for game in games {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&game.name).strong());
                        ui.label(&game.state);
                        ui.label(&game.store);
                    });
                    ui.monospace(game.executable.display().to_string());
                    ui.label(RichText::new(game.id.to_string()).weak());
                });
            }
        });
    }

    fn about_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("About Bottles (Rust/egui)");
        ui.label(format!("Version {APP_VERSION}"));
        ui.label(format!("Application ID: {APP_ID}"));
        ui.hyperlink_to("Documentation", DOC_URL);
        ui.separator();
        ui.heading("Keyboard shortcuts");
        for (shortcut, action) in [
            ("Ctrl+Q / Ctrl+W", "Quit"),
            ("Ctrl+I", "Import"),
            ("Ctrl+J", "Journal"),
            ("Ctrl+,", "Preferences"),
            ("F1", "Help"),
            ("Ctrl+N", "New bottle"),
        ] {
            ui.horizontal(|ui| {
                ui.monospace(shortcut);
                ui.label(action);
            });
        }
        ui.separator();
        ui.label("This Rust application reads and writes the same bottle.yml, library.yml, journal.yml, data.yml, and UMU game.yml files as the Python implementation.");
    }

    fn confirm_delete_window(&mut self, ctx: &egui::Context) {
        if !self.state.confirm_delete {
            return;
        }
        let Some(bottle) = self.state.selected_bottle() else {
            self.state.confirm_delete = false;
            return;
        };
        let mut open = self.state.confirm_delete;
        egui::Window::new("Delete bottle")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Permanently delete '{}' and its Wine prefix?",
                    bottle.name()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.state.confirm_delete = false;
                    }
                    if ui.button("Delete").clicked() {
                        self.state.delete_selected();
                        self.state.confirm_delete = false;
                    }
                });
            });
        self.state.confirm_delete = open;
    }
}

impl eframe::App for BottlesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.configure(ctx);
        egui::SidePanel::left("navigation")
            .resizable(true)
            .default_width(210.0)
            .show(ctx, |ui| self.sidebar(ui));
        egui::TopBottomPanel::top("search").show(ctx, |ui| self.top_bar(ui));
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.bottom_bar(ui));
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.state.screen {
                    Screen::Library => self.library_screen(ui),
                    Screen::Bottles => self.bottles_screen(ui),
                    Screen::Details => self.details_screen(ui),
                    Screen::Components => self.components_screen(ui),
                    Screen::Importer => self.importer_screen(ui),
                    Screen::Journal => self.journal_screen(ui),
                    Screen::Health => self.health_screen(ui),
                    Screen::Preferences => self.preferences_screen(ui),
                    Screen::Parity => self.parity_screen(ui),
                    Screen::Umu => self.umu_screen(ui),
                    Screen::About => self.about_screen(ui),
                });
        });
        self.confirm_delete_window(ctx);
    }
}

fn labeled_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
    });
}

fn status_badge(ui: &mut egui::Ui, status: ParityStatus) {
    let (text, color) = match status {
        ParityStatus::Ported => ("ported", egui::Color32::DARK_GREEN),
        ParityStatus::Partial => ("partial", egui::Color32::from_rgb(140, 100, 0)),
        ParityStatus::InventoryOnly => ("inventory", egui::Color32::GRAY),
        ParityStatus::ReferenceOnly => ("reference", egui::Color32::DARK_RED),
    };
    ui.colored_label(color, text);
}

fn period_label(period: JournalPeriod) -> &'static str {
    match period {
        JournalPeriod::All => "all",
        JournalPeriod::Today => "today",
        JournalPeriod::Yesterday => "yesterday",
        JournalPeriod::Week => "week",
        JournalPeriod::Month => "month",
    }
}

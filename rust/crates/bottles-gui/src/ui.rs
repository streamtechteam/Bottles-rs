use crate::catalog::catalogs;
use crate::state::{AppState, Dialog, MainTab, View};
use crate::theme::{self, Icon, Palette};
use bottles_core::health::HealthChecker;
use bottles_core::journal::JournalPeriod;
use bottles_core::parity::{inventory, ParityStatus};
use bottles_core::state::TaskManager;
use bottles_core::wine::WineTool;
use bottles_core::{APP_ID, APP_NAME, APP_VERSION, DOC_URL};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin,
    RichText, ScrollArea, Sense, Shape, Stroke, Ui, ViewportCommand,
};
use serde_yaml::Value;

pub struct BottlesApp {
    pub state: AppState,
}

impl BottlesApp {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl eframe::App for BottlesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let palette = theme::apply(
            ctx,
            self.state.context.settings.dark_theme,
            self.state.context.settings.font_scale,
        );
        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("header")
            .exact_height(48.0)
            .frame(Frame::new().fill(palette.header))
            .show(ctx, |ui| self.header(ui, &palette, ctx));

        if self.state.view == View::Main && self.state.search_visible {
            egui::TopBottomPanel::top("search")
                .exact_height(46.0)
                .frame(Frame::new().fill(palette.background))
                .show(ctx, |ui| self.search_bar(ui, &palette));
        }

        egui::TopBottomPanel::bottom("status")
            .exact_height(28.0)
            .frame(Frame::new().fill(palette.header))
            .show(ctx, |ui| self.status_bar(ui, &palette));

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(palette.background)
                    .inner_margin(Margin::same(18)),
            )
            .show(ctx, |ui| self.content(ui, &palette, ctx));

        self.menus(ctx, &palette);
        self.dialogs(ctx, &palette);
    }
}

impl BottlesApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            return;
        }
        let state = &mut self.state;
        if state.show_add_menu
            || state.show_main_menu
            || state.show_power_menu
            || state.show_context_menu
        {
            state.close_menus();
        } else if state.create_dialog
            || state.run_dialog
            || state.shortcut_dialog
            || state.rename_dialog
            || state.duplicate_dialog
            || state.delete_dialog
            || state.backup_dialog
            || state.launch_options_dialog
            || state.env_dialog
            || state.registry_dialog
            || state.help_dialog
            || state.about_dialog
        {
            state.create_dialog = false;
            state.run_dialog = false;
            state.shortcut_dialog = false;
            state.rename_dialog = false;
            state.duplicate_dialog = false;
            state.delete_dialog = false;
            state.backup_dialog = false;
            state.launch_options_dialog = false;
            state.env_dialog = false;
            state.registry_dialog = false;
            state.help_dialog = false;
            state.about_dialog = false;
        } else if state.view != View::Main {
            state.back();
        }
    }

    fn header(&mut self, ui: &mut Ui, palette: &Palette, ctx: &egui::Context) {
        let rect = ui.max_rect();
        let painter = ui.painter();
        painter.line_segment(
            [
                pos2(rect.left(), rect.bottom() - 0.5),
                pos2(rect.right(), rect.bottom() - 0.5),
            ],
            Stroke::new(1.0_f32, palette.border),
        );
        ui.add_space(7.0);
        let state = &mut self.state;
        ui.columns(3, |columns| {
            columns[0].horizontal(|ui| {
                if state.view == View::Main {
                    if theme::icon_button(ui, palette, Icon::Plus, "Add").clicked() {
                        state.show_add_menu = !state.show_add_menu;
                        state.show_main_menu = false;
                    }
                } else if theme::icon_button(ui, palette, Icon::Back, "Go Back").clicked() {
                    state.back();
                }
            });

            columns[1].vertical_centered(|ui| {
                if state.view == View::Main {
                    ui.horizontal(|ui| {
                        let (logo_rect, _) =
                            ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                        draw_bottle_logo(ui.painter(), logo_rect, palette.text);
                        ui.add_space(2.0);
                        let bottles = ui.add(
                            egui::Button::new("Bottles")
                                .selected(state.main_tab == MainTab::Bottles)
                                .corner_radius(CornerRadius::same(8)),
                        );
                        let library = ui.add(
                            egui::Button::new("Library")
                                .selected(state.main_tab == MainTab::Library)
                                .corner_radius(CornerRadius::same(8)),
                        );
                        if bottles.clicked() {
                            state.main_tab = MainTab::Bottles;
                        }
                        if library.clicked() {
                            state.main_tab = MainTab::Library;
                        }
                    });
                } else {
                    ui.label(RichText::new(state.view.title()).strong().size(15.0));
                }
            });

            columns[2].with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::icon_button(ui, palette, Icon::Close, "Close").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                if state.view == View::Main {
                    if theme::icon_button(ui, palette, Icon::Menu, "Main Menu").clicked() {
                        state.show_main_menu = !state.show_main_menu;
                        state.show_add_menu = false;
                    }
                    if theme::icon_button(ui, palette, Icon::Search, "Search").clicked() {
                        state.search_visible = !state.search_visible;
                    }
                    if theme::icon_button(ui, palette, Icon::Heart, "Support Bottles").clicked() {
                        state.about_dialog = true;
                    }
                } else {
                    if theme::icon_button(ui, palette, Icon::More, "Secondary Menu").clicked() {
                        state.show_context_menu = !state.show_context_menu;
                    }
                    if state.view == View::Details
                        && theme::icon_button(ui, palette, Icon::Power, "Power Options").clicked()
                    {
                        state.show_power_menu = !state.show_power_menu;
                    }
                    if matches!(state.view, View::Installers | View::Dependencies)
                        && theme::icon_button(ui, palette, Icon::Search, "Search").clicked()
                    {
                        state.search_visible = !state.search_visible;
                    }
                }
            });
        });
    }

    fn search_bar(&mut self, ui: &mut Ui, palette: &Palette) {
        let state = &mut self.state;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Search").color(palette.subtext));
            ui.add(
                egui::TextEdit::singleline(&mut state.search)
                    .hint_text("Search your bottles or library…")
                    .desired_width(320.0),
            );
            if theme::secondary_button(ui, palette, "Clear").clicked() {
                state.search.clear();
            }
        });
    }

    fn status_bar(&mut self, ui: &mut Ui, palette: &Palette) {
        let state = &self.state;
        ui.horizontal(|ui| {
            if let Some(error) = &state.error {
                ui.colored_label(palette.danger, format!("● {error}"));
            } else if let Some(toast) = &state.toast {
                ui.colored_label(palette.success, format!("● {toast}"));
            } else {
                ui.label(RichText::new("Ready").color(palette.subtext).size(11.5));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} bottles", state.bottles.len()))
                        .color(palette.subtext)
                        .size(11.5),
                );
            });
        });
    }

    fn content(&mut self, ui: &mut Ui, palette: &Palette, ctx: &egui::Context) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.state.view {
                View::Main => match self.state.main_tab {
                    MainTab::Bottles => self.bottles_view(ui, palette),
                    MainTab::Library => self.library_view(ui, palette),
                },
                View::Details => self.details_view(ui, palette),
                View::Settings => self.settings_view(ui, palette),
                View::Dependencies => self.dependencies_view(ui, palette),
                View::Installers => self.installers_view(ui, palette),
                View::RegistryRules => self.registry_rules_view(ui, palette),
                View::Snapshots => self.snapshots_view(ui, palette),
                View::TaskManager => self.task_manager_view(ui, palette),
                View::Importer => self.importer_view(ui, palette),
                View::Journal => self.journal_view(ui, palette),
                View::Preferences => self.preferences_view(ui, palette),
                View::Components => self.components_view(ui, palette),
                View::Health => self.health_view(ui, palette),
                View::Parity => self.parity_view(ui, palette),
                View::Umu => self.umu_view(ui, palette),
                View::About => self.about_view(ui, palette),
                View::Help => self.help_view(ui, palette),
            });
        let _ = ctx;
    }

    fn bottles_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(720.0);
            let state = &mut self.state;
            let bottles = state.bottles.clone();
            if bottles.is_empty() {
                ui.add_space(72.0);
                let (rect, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                draw_bottle_logo(ui.painter(), rect, palette.subtext);
                ui.add_space(18.0);
                ui.label(RichText::new("Bottles").strong().size(28.0));
                ui.label(
                    RichText::new("Create a bottle to run Windows software.")
                        .color(palette.subtext),
                );
                ui.add_space(14.0);
                if theme::pill_button(ui, palette, "Create a New Bottle…", 220.0).clicked() {
                    state.create_dialog = true;
                }
                return;
            }

            let query = state.search.to_ascii_lowercase();
            let local: Vec<_> = bottles
                .iter()
                .filter(|bottle| bottle.config.environment != "Steam")
                .filter(|bottle| {
                    query.is_empty() || bottle.name().to_ascii_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            let steam: Vec<_> = bottles
                .iter()
                .filter(|bottle| bottle.config.environment == "Steam")
                .filter(|bottle| {
                    query.is_empty() || bottle.name().to_ascii_lowercase().contains(&query)
                })
                .cloned()
                .collect();

            if !local.is_empty() {
                theme::section_title(ui, palette, "Your Bottles");
                theme::card(ui, palette, |ui| {
                    for bottle in &local {
                        let accessory = format!("{}   ›", bottle.config.environment);
                        let response = theme::clickable_row(
                            ui,
                            palette,
                            bottle.name(),
                            &bottle.config.runner,
                            &accessory,
                        );
                        if response.clicked() {
                            state.open_bottle(bottle.name().to_owned());
                        }
                        response.context_menu(|ui| bottle_context_menu(ui, state, bottle.name()));
                    }
                });
            }

            if !steam.is_empty() {
                theme::section_title(ui, palette, "Steam Proton");
                theme::card(ui, palette, |ui| {
                    for bottle in &steam {
                        let response = theme::clickable_row(
                            ui,
                            palette,
                            bottle.name(),
                            &bottle.config.runner,
                            "Steam   ›",
                        );
                        if response.clicked() {
                            state.open_bottle(bottle.name().to_owned());
                        }
                        response.context_menu(|ui| bottle_context_menu(ui, state, bottle.name()));
                    }
                });
            }

            if !state.umu_games.is_empty() {
                theme::section_title(ui, palette, "UMU Prefixes");
                let games = state.umu_games.clone();
                theme::card(ui, palette, |ui| {
                    for game in &games {
                        let response =
                            theme::clickable_row(ui, palette, &game.name, &game.proton, "UMU   ›");
                        if response.clicked() {
                            state.navigate(View::Umu);
                        }
                    }
                });
            }

            if local.is_empty() && steam.is_empty() && !query.is_empty() {
                ui.add_space(48.0);
                ui.label(RichText::new("No Results Found").strong().size(20.0));
                ui.label(RichText::new("Try a different search.").color(palette.subtext));
            }
        });
    }

    fn library_view(&mut self, ui: &mut Ui, palette: &Palette) {
        let state = &mut self.state;
        let query = state.search.to_ascii_lowercase();
        let entries: Vec<(String, String, String)> = state
            .context
            .library
            .entries
            .iter()
            .filter_map(|(id, entry)| {
                let name = field(entry, "name").unwrap_or_else(|| "Untitled".to_owned());
                let bottle = field(entry, "bottle").unwrap_or_default();
                if !query.is_empty() && !name.to_ascii_lowercase().contains(&query) {
                    return None;
                }
                Some((id.clone(), name, bottle))
            })
            .collect();

        if entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(96.0);
                ui.label(RichText::new("No Results Found").strong().size(22.0));
                ui.label(
                    RichText::new("Programs you add to the Library will appear here.")
                        .color(palette.subtext),
                );
            });
            return;
        }

        ui.horizontal_wrapped(|ui| {
            for (id, name, bottle) in entries {
                theme::card(ui, palette, |ui| {
                    ui.set_min_size(vec2(180.0, 252.0));
                    let (rect, _) = ui.allocate_exact_size(vec2(160.0, 190.0), Sense::hover());
                    let color = cover_color(&name, palette);
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(10), color);
                    let initials = name
                        .split_whitespace()
                        .take(2)
                        .map(|word| word.chars().next().unwrap_or('?'))
                        .collect::<String>()
                        .to_uppercase();
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        initials,
                        FontId::proportional(34.0),
                        Color32::WHITE,
                    );
                    ui.add_space(6.0);
                    ui.label(RichText::new(&name).strong());
                    ui.label(RichText::new(&bottle).color(palette.subtext).size(11.5));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Launch").clicked() {
                            state.set_toast(format!("Open {name} from its bottle to launch it"));
                        }
                        if ui.button("Remove").clicked() {
                            state.remove_library_entry(&id);
                        }
                    });
                });
            }
        });
    }

    fn details_view(&mut self, ui: &mut Ui, palette: &Palette) {
        let Some(bottle) = self.state.selected_bottle() else {
            ui.label("Create or select a bottle first.");
            return;
        };
        ui.vertical_centered(|ui| {
            ui.set_max_width(620.0);
            ui.add_space(12.0);
            ui.label(RichText::new(&bottle.config.name).strong().size(24.0));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                theme::badge(ui, palette, &nice_arch(&bottle.config.arch));
                ui.add_space(2.0);
                theme::badge(ui, palette, &bottle.config.environment);
                ui.add_space(2.0);
                theme::badge(ui, palette, &bottle.config.runner);
            });
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                if theme::pill_button(ui, palette, "▶  Run Executable…", 190.0).clicked() {
                    self.state.run_dialog = true;
                }
                if theme::icon_button(ui, palette, Icon::Gear, "Launch Options").clicked() {
                    self.state.launch_options_dialog = true;
                }
            });
            ui.add_space(20.0);

            theme::section_title(ui, palette, "Programs");
            theme::card(ui, palette, |ui| {
                let programs: Vec<_> = bottle
                    .config
                    .external_programs
                    .values()
                    .filter(|program| {
                        self.state.show_hidden_programs || !program.removed.unwrap_or(false)
                    })
                    .cloned()
                    .collect();
                if programs.is_empty() {
                    ui.label(
                        RichText::new(
                            "Click \"Run Executable…\" to run a program, \"Add Shortcuts…\" to add one, or \"Install Programs…\" to install programs curated by the community.",
                        )
                        .color(palette.subtext),
                    );
                } else {
                    for program in &programs {
                        ui.horizontal(|ui| {
                            if theme::icon_button(ui, palette, Icon::Play, "Launch").clicked() {
                                self.state.launch_program(program);
                            }
                            ui.vertical(|ui| {
                                ui.label(RichText::new(program.display_name()).strong());
                                ui.label(
                                    RichText::new(program.path.clone().unwrap_or_default())
                                        .color(palette.subtext)
                                        .size(11.0),
                                );
                            });
                        });
                        ui.separator();
                    }
                }
                ui.columns(2, |columns| {
                    if columns[0]
                        .add(egui::Button::new("+  Add Shortcuts…").frame(false))
                        .clicked()
                    {
                        self.state.shortcut_dialog = true;
                    }
                    if columns[1]
                        .add(egui::Button::new("↓  Install Programs…").frame(false))
                        .clicked()
                    {
                        self.state.navigate(View::Installers);
                    }
                });
            });

            theme::section_title(ui, palette, "Options");
            theme::card(ui, palette, |ui| {
                let rows = [
                    (View::Settings, "Settings", "Configure bottle settings."),
                    (
                        View::Dependencies,
                        "Dependencies",
                        "Install dependencies for programs.",
                    ),
                    (
                        View::RegistryRules,
                        "Registry Rules",
                        "Manage reusable registry tweaks.",
                    ),
                    (
                        View::Snapshots,
                        "Snapshots",
                        "Create and manage bottle states.",
                    ),
                    (
                        View::TaskManager,
                        "Task Manager",
                        "Manage running programs.",
                    ),
                ];
                for (view, title, subtitle) in rows {
                    if theme::clickable_row(ui, palette, title, subtitle, "›").clicked() {
                        self.state.navigate(view);
                    }
                }
            });

            theme::section_title(ui, palette, "Tools");
            theme::card(ui, palette, |ui| {
                let tools = [
                    ("Command Line", WineTool::Cmd),
                    ("Wine Configuration", WineTool::WineCfg),
                    ("Registry Editor", WineTool::Regedit),
                    ("File Explorer", WineTool::Explorer),
                    ("Task Manager", WineTool::Taskmgr),
                    ("Control Panel", WineTool::Control),
                ];
                for (label, tool) in tools {
                    if theme::clickable_row(ui, palette, label, "", "›").clicked() {
                        self.state.run_wine_tool(tool, &[]);
                    }
                }
            });
        });
    }

    fn settings_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            let Some(bottle) = state.selected_bottle() else {
                ui.label("Select a bottle first.");
                return;
            };
            theme::section_title(ui, palette, "Name");
            theme::card(ui, palette, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&bottle.config.name).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::icon_button(ui, palette, Icon::Gear, "Rename").clicked() {
                            state.rename_dialog = true;
                        }
                    });
                });
            });

            theme::section_title(ui, palette, "Components");
            theme::card(ui, palette, |ui| {
                component_combo(
                    ui,
                    palette,
                    "Runner",
                    "The version of the Wine compatibility layer.",
                    "runners",
                    &mut state.edit_runner,
                );
                component_combo(
                    ui,
                    palette,
                    "D7VK",
                    "Improve Direct3D 3/5/6/7 compatibility.",
                    "d7vk",
                    &mut state.edit_d7vk,
                );
                component_combo(
                    ui,
                    palette,
                    "DXVK",
                    "Improve Direct3D 8/9/10/11 compatibility by translating it to Vulkan.",
                    "dxvk",
                    &mut state.edit_dxvk_version,
                );
                component_combo(
                    ui,
                    palette,
                    "VKD3D",
                    "Improve Direct3D 12 compatibility by translating it to Vulkan.",
                    "vkd3d",
                    &mut state.edit_vkd3d_version,
                );
                component_combo(
                    ui,
                    palette,
                    "DXVK NVAPI",
                    "NVIDIA DLSS and NVAPI support.",
                    "nvapi",
                    &mut state.edit_nvapi,
                );
                component_combo(
                    ui,
                    palette,
                    "LatencyFleX",
                    "Increase responsiveness. Can be detected by some anti-cheat software.",
                    "latencyflex",
                    &mut state.edit_latencyflex,
                );
            });

            theme::section_title(ui, palette, "Display");
            theme::card(ui, palette, |ui| {
                setting_switch(
                    ui,
                    palette,
                    "FidelityFX Super Resolution",
                    "Increase performance at the expense of visuals. Only works on Vulkan.",
                    &mut state.edit_params.fsr,
                );
                setting_switch(
                    ui,
                    palette,
                    "Discrete Graphics",
                    "Use the discrete graphics card to increase performance.",
                    &mut state.edit_params.discrete_gpu,
                );
                setting_switch(
                    ui,
                    palette,
                    "Virtual Desktop",
                    "Run programs inside a virtual desktop window.",
                    &mut state.edit_params.virtual_desktop,
                );
                setting_switch(
                    ui,
                    palette,
                    "Wayland (Experimental)",
                    "Force the Wayland driver when available.",
                    &mut state.edit_params.wayland,
                );
                setting_switch(
                    ui,
                    palette,
                    "Post-Processing Effects",
                    "Add post-processing effects using vkBasalt. Only works on Vulkan.",
                    &mut state.edit_params.vkbasalt,
                );
            });

            theme::section_title(ui, palette, "System");
            theme::card(ui, palette, |ui| {
                setting_switch(
                    ui,
                    palette,
                    "MangoHud",
                    "Display a performance overlay.",
                    &mut state.edit_params.mangohud,
                );
                setting_switch(
                    ui,
                    palette,
                    "Gamescope",
                    "Use the Gamescope compositor.",
                    &mut state.edit_params.gamescope,
                );
                setting_switch(
                    ui,
                    palette,
                    "GameMode",
                    "Request Feral GameMode optimizations.",
                    &mut state.edit_params.gamemode,
                );
                setting_switch(
                    ui,
                    palette,
                    "Winebridge",
                    "Enable the Bottles Winebridge integration.",
                    &mut state.edit_params.winebridge,
                );
                setting_switch(
                    ui,
                    palette,
                    "Sandbox",
                    "Run programs inside a restricted environment.",
                    &mut state.edit_params.sandbox,
                );
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::pill_button(ui, palette, "Save Settings", 140.0).clicked() {
                    state.save_selected_configuration();
                }
                if theme::secondary_button(ui, palette, "Environment Variables").clicked() {
                    state.env_dialog = true;
                }
                if theme::secondary_button(ui, palette, "Repair Bottle").clicked() {
                    state.repair_selected();
                }
            });
        });
    }

    fn dependencies_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(720.0);
            ui.label(
                RichText::new(
                    "Dependencies are resources that improve compatibility of Windows software.",
                )
                .color(palette.subtext),
            );
            ui.label(
                RichText::new(
                    "Files on this page are provided by third parties under a proprietary license. By installing them, you agree with their respective licensing terms.",
                )
                .color(palette.subtext),
            );
            ui.add_space(8.0);
            let state = &mut self.state;
            ui.add(
                egui::TextEdit::singleline(&mut state.dependency_query)
                    .hint_text("Search for dependencies…")
                    .desired_width(420.0),
            );
            ui.add_space(8.0);
            let query = state.dependency_query.to_ascii_lowercase();
            let dependencies = catalogs().dependencies.clone();
            let visible: Vec<_> = dependencies
                .iter()
                .filter(|item| {
                    query.is_empty()
                        || item.description.to_ascii_lowercase().contains(&query)
                        || item.id.to_ascii_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            theme::card(ui, palette, |ui| {
                for dependency in visible.iter().take(80) {
                    let subtitle = format!("{} · {}", dependency.id, dependency.category);
                    let response = theme::clickable_row(
                        ui,
                        palette,
                        &dependency.description,
                        &subtitle,
                        "",
                    );
                    if response.clicked() {
                        state.set_toast(format!(
                            "Dependency installation is still provided by the Python application: {}",
                            dependency.id
                        ));
                    }
                }
            });
        });
    }

    fn installers_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(720.0);
            ui.label(
                RichText::new("Install programs curated by our community.").color(palette.subtext),
            );
            ui.label(
                RichText::new(
                    "Files on this page are provided by third parties under a proprietary license. By installing them, you agree with their respective licensing terms.",
                )
                .color(palette.subtext),
            );
            ui.add_space(8.0);
            let state = &mut self.state;
            ui.add(
                egui::TextEdit::singleline(&mut state.installer_query)
                    .hint_text("Search for Programs…")
                    .desired_width(420.0),
            );
            ui.add_space(8.0);
            let bottle = state.selected_bottle();
            let arch = bottle
                .as_ref()
                .map(|value| value.config.arch.clone())
                .unwrap_or_else(|| "win64".to_owned());
            let runner = bottle
                .as_ref()
                .map(|value| value.config.runner.clone())
                .unwrap_or_default();
            let include_unstable = state.context.settings.release_candidate;
            let query = state.installer_query.to_ascii_lowercase();
            let visible: Vec<_> = catalogs()
                .installers_for(&arch, &runner, include_unstable)
                .into_iter()
                .filter(|installer| {
                    query.is_empty()
                        || installer.name.to_ascii_lowercase().contains(&query)
                        || installer.description.to_ascii_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            if visible.is_empty() {
                ui.add_space(48.0);
                ui.label(RichText::new("No Installers Found").strong().size(20.0));
                ui.label(
                    RichText::new(
                        "The repository is unreachable or no installer is compatible with this bottle.",
                    )
                    .color(palette.subtext),
                );
                return;
            }
            theme::card(ui, palette, |ui| {
                for installer in visible {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&installer.name).strong());
                            ui.label(
                                RichText::new(&installer.description)
                                    .color(palette.subtext)
                                    .size(11.5),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if theme::icon_button(ui, palette, Icon::More, "Installer options")
                                .clicked()
                            {
                                state.set_toast(format!(
                                    "Installer execution is still provided by the Python application: {}",
                                    installer.id
                                ));
                            }
                            if theme::icon_button(ui, palette, Icon::Download, "Install").clicked() {
                                state.set_toast(format!(
                                    "Installer execution is still provided by the Python application: {}",
                                    installer.id
                                ));
                            }
                            theme::grade_badge(ui, palette, &installer.grade);
                        });
                    });
                    ui.separator();
                }
            });
        });
    }

    fn registry_rules_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            let Some(bottle) = state.selected_bottle() else {
                ui.label("Select a bottle first.");
                return;
            };
            if bottle.config.registry_rules.is_empty() {
                ui.add_space(48.0);
                ui.label(RichText::new("No rules yet").strong().size(20.0));
                ui.label(
                    RichText::new("Create a registry rule to reuse your tweaks across bottles.")
                        .color(palette.subtext),
                );
            } else {
                theme::card(ui, palette, |ui| {
                    for rule in &bottle.config.registry_rules {
                        theme::clickable_row(ui, palette, &rule.name, &rule.description, "");
                    }
                });
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if theme::pill_button(ui, palette, "New Rule", 120.0).clicked() {
                    state.registry_dialog = true;
                }
                if theme::secondary_button(ui, palette, "Apply Rules").clicked() {
                    state.apply_registry_rules();
                }
            });
        });
    }

    fn snapshots_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            let Some(bottle) = state.selected_bottle() else {
                ui.label("Select a bottle first.");
                return;
            };
            ui.label(
                RichText::new(format!("Branch: {}", bottle.config.state))
                    .color(palette.subtext),
            );
            ui.add_space(8.0);
            if bottle.config.state == 0 {
                ui.add_space(48.0);
                ui.label(RichText::new("No Snapshots Found").strong().size(20.0));
                ui.label(
                    RichText::new(
                        "Create your first snapshot to start saving states of your preferences.",
                    )
                    .color(palette.subtext),
                );
            } else {
                theme::card(ui, palette, |ui| {
                    theme::clickable_row(ui, palette, "main", "Current bottle state", "");
                });
            }
            ui.add_space(12.0);
            ui.label(
                RichText::new(
                    "Incremental FVS snapshots are still implemented by the Python application. Full backups are available from the bottle menu.",
                )
                .color(palette.subtext),
            );
        });
    }

    fn task_manager_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(720.0);
            let tasks = TaskManager::list();
            if tasks.is_empty() {
                ui.add_space(48.0);
                ui.label(RichText::new("No running tasks").strong().size(20.0));
                ui.label(
                    RichText::new("Programs launched from Bottles will appear here.")
                        .color(palette.subtext),
                );
                return;
            }
            theme::card(ui, palette, |ui| {
                for task in tasks {
                    theme::clickable_row(
                        ui,
                        palette,
                        &task.title,
                        &format!("{} · {}", task.subtitle, task.status.as_str()),
                        "",
                    );
                }
            });
        });
    }

    fn importer_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            theme::section_title(ui, palette, "Found Wine prefixes");
            if state.importer_candidates.is_empty() {
                ui.label(
                    RichText::new("No Wine prefixes were found in the usual locations.")
                        .color(palette.subtext),
                );
            } else {
                let candidates = state.importer_candidates.clone();
                theme::card(ui, palette, |ui| {
                    for (index, candidate) in candidates.iter().enumerate() {
                        if theme::clickable_row(
                            ui,
                            palette,
                            &candidate.name,
                            &format!("{} · {}", candidate.manager, candidate.path.display()),
                            "Import",
                        )
                        .clicked()
                        {
                            state.import_candidate(index);
                        }
                    }
                });
            }
            theme::section_title(ui, palette, "Manual path");
            theme::card(ui, palette, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut state.importer_manual_path)
                            .hint_text("Path to a Wine prefix"),
                    );
                    if theme::secondary_button(ui, palette, "Search").clicked() {
                        state.refresh_all();
                        state.set_toast("Refreshed Wine prefix search");
                    }
                });
            });
        });
    }

    fn journal_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(760.0);
            let state = &mut self.state;
            ui.horizontal(|ui| {
                ui.label("Period:");
                egui::ComboBox::from_id_salt("journal-period")
                    .selected_text(period_label(state.journal_period))
                    .show_ui(ui, |ui| {
                        for value in JournalPeriod::ALL {
                            let period = JournalPeriod::from_str(value);
                            ui.selectable_value(&mut state.journal_period, period, *value);
                        }
                    });
                if theme::secondary_button(ui, palette, "Refresh").clicked() {
                    state.refresh_all();
                }
            });
            ui.add_space(8.0);
            let mut text = state
                .context
                .journal
                .plain(state.journal_period)
                .unwrap_or_else(|error| format!("Could not read the journal: {error}"));
            theme::card(ui, palette, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(18)
                        .font(egui::TextStyle::Monospace),
                );
            });
        });
    }

    fn preferences_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            theme::section_title(ui, palette, "Appearance");
            theme::card(ui, palette, |ui| {
                setting_switch(
                    ui,
                    palette,
                    "Dark Theme",
                    "Use the dark interface theme.",
                    &mut state.context.settings.dark_theme,
                );
                ui.horizontal(|ui| {
                    ui.label("Font scale");
                    ui.add(egui::Slider::new(
                        &mut state.context.settings.font_scale,
                        0.8..=2.0,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Startup view");
                    egui::ComboBox::from_id_salt("startup-view")
                        .selected_text(if state.context.settings.startup_view == "page_bottles" {
                            "Bottles"
                        } else {
                            "Library"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut state.context.settings.startup_view,
                                "page_bottles".to_owned(),
                                "Bottles",
                            );
                            ui.selectable_value(
                                &mut state.context.settings.startup_view,
                                "page_library".to_owned(),
                                "Library",
                            );
                        });
                });
            });
            theme::section_title(ui, palette, "Behavior");
            theme::card(ui, palette, |ui| {
                setting_switch(
                    ui,
                    palette,
                    "Notifications",
                    "Show desktop notifications.",
                    &mut state.context.settings.notifications,
                );
                setting_switch(
                    ui,
                    palette,
                    "Playtime tracking",
                    "Track time spent in programs.",
                    &mut state.context.settings.playtime_enabled,
                );
                setting_switch(
                    ui,
                    palette,
                    "Release candidate components",
                    "Include unstable components and installers.",
                    &mut state.context.settings.release_candidate,
                );
                setting_switch(
                    ui,
                    palette,
                    "Force offline mode",
                    "Do not contact component repositories.",
                    &mut state.context.settings.force_offline,
                );
            });
            ui.add_space(10.0);
            if theme::pill_button(ui, palette, "Save Preferences", 160.0).clicked() {
                state.save_preferences();
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, palette, "System Health").clicked() {
                    state.navigate(View::Health);
                }
                if theme::secondary_button(ui, palette, "Components").clicked() {
                    state.navigate(View::Components);
                }
                if theme::secondary_button(ui, palette, "Parity Report").clicked() {
                    state.navigate(View::Parity);
                }
            });
        });
    }

    fn components_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(760.0);
            let state = &mut self.state;
            ui.add(
                egui::TextEdit::singleline(&mut state.component_query)
                    .hint_text("Search installed and available components…")
                    .desired_width(420.0),
            );
            ui.add_space(8.0);
            let query = state.component_query.to_ascii_lowercase();
            theme::section_title(ui, palette, "Installed components");
            theme::card(ui, palette, |ui| {
                if state.components.is_empty() {
                    ui.label(RichText::new("No components installed.").color(palette.subtext));
                } else {
                    for (kind, versions) in &state.components {
                        if !query.is_empty() && !kind.to_ascii_lowercase().contains(&query) {
                            continue;
                        }
                        theme::clickable_row(ui, palette, kind, &versions.join(", "), "");
                    }
                }
            });
            theme::section_title(ui, palette, "Available components");
            let catalog = catalogs();
            theme::card(ui, palette, |ui| {
                for category in ["runners", "dxvk", "vkd3d", "nvapi", "d7vk", "latencyflex"] {
                    let names = catalog.component_names(category);
                    if names.is_empty() {
                        continue;
                    }
                    theme::clickable_row(
                        ui,
                        palette,
                        category,
                        &format!("{} versions in the pinned catalog", names.len()),
                        "",
                    );
                }
            });
        });
    }

    fn health_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(760.0);
            let mut yaml = HealthChecker::collect()
                .to_yaml()
                .unwrap_or_else(|error| format!("Could not collect health information: {error}"));
            theme::card(ui, palette, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut yaml)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .font(egui::TextStyle::Monospace),
                );
            });
        });
    }

    fn parity_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(860.0);
            let state = &mut self.state;
            let (ported, partial, inventory_only, reference_only) = state.parity_counts();
            ui.horizontal(|ui| {
                theme::badge(ui, palette, &format!("{ported} ported"));
                theme::badge(ui, palette, &format!("{partial} partial"));
                theme::badge(ui, palette, &format!("{inventory_only} inventory"));
                theme::badge(ui, palette, &format!("{reference_only} reference"));
            });
            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.parity_search)
                    .hint_text("Search the source-derived feature inventory…")
                    .desired_width(480.0),
            );
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.parity_status, None, "All");
                for candidate in ParityStatus::ALL {
                    ui.selectable_value(
                        &mut state.parity_status,
                        Some(*candidate),
                        candidate.label(),
                    );
                }
            });
            ui.add_space(8.0);
            let query = state.parity_search.clone();
            let items = inventory();
            let visible: Vec<_> = items
                .iter()
                .filter(|item| item.matches(&query))
                .filter(|item| {
                    state
                        .parity_status
                        .is_none_or(|status| item.status == status)
                })
                .collect();
            theme::card(ui, palette, |ui| {
                for item in visible.iter().take(200) {
                    theme::clickable_row(
                        ui,
                        palette,
                        &format!("{} · {}", item.kind, item.id),
                        &format!("{} · {}", item.status.label(), item.source),
                        "",
                    );
                }
            });
        });
    }

    fn umu_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(680.0);
            let state = &mut self.state;
            theme::section_title(ui, palette, "UMU games");
            if state.umu_games.is_empty() {
                ui.label(RichText::new("No UMU games configured.").color(palette.subtext));
            } else {
                let games = state.umu_games.clone();
                theme::card(ui, palette, |ui| {
                    for game in games {
                        theme::clickable_row(
                            ui,
                            palette,
                            &game.name,
                            &format!("{} · {}", game.proton, game.state),
                            "",
                        );
                    }
                });
            }
            theme::section_title(ui, palette, "Add a game");
            theme::card(ui, palette, |ui| {
                labeled_text(ui, "Name", &mut state.umu_name);
                labeled_text(ui, "Executable", &mut state.umu_executable);
                labeled_text(ui, "Proton", &mut state.umu_proton);
                if theme::pill_button(ui, palette, "Add Game", 120.0).clicked() {
                    state.add_umu_game();
                }
            });
        });
    }

    fn about_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(560.0);
            ui.add_space(32.0);
            let (rect, _) = ui.allocate_exact_size(vec2(88.0, 88.0), Sense::hover());
            draw_bottle_logo(ui.painter(), rect, palette.text);
            ui.add_space(12.0);
            ui.label(RichText::new(APP_NAME).strong().size(24.0));
            ui.label(RichText::new(format!("Version {APP_VERSION}")).color(palette.subtext));
            ui.label(RichText::new(APP_ID).monospace().color(palette.subtext));
            ui.add_space(12.0);
            ui.hyperlink_to("Documentation", format!("https://{DOC_URL}"));
            ui.add_space(18.0);
            ui.label(
                RichText::new(
                    "This Rust/egui front end reads and writes the same Bottles data files as the Python application. Network repositories, dependency recipes, and installer execution still rely on the Python implementation.",
                )
                .color(palette.subtext),
            );
        });
    }

    fn help_view(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical_centered(|ui| {
            ui.set_max_width(560.0);
            theme::section_title(ui, palette, "Keyboard Shortcuts");
            theme::card(ui, palette, |ui| {
                for (shortcut, description) in [
                    ("Ctrl + F", "Search bottles and library"),
                    ("Esc", "Close a dialog or go back"),
                    ("F1", "Open this help"),
                    ("Ctrl + Q", "Close Bottles"),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(shortcut).monospace().strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(description);
                        });
                    });
                    ui.separator();
                }
            });
        });
    }

    fn menus(&mut self, ctx: &egui::Context, palette: &Palette) {
        let state = &mut self.state;
        if state.show_add_menu {
            egui::Window::new("Add")
                .id(egui::Id::new("add-menu"))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::LEFT_TOP, vec2(18.0, 54.0))
                .frame(theme::card_frame(palette, 12))
                .show(ctx, |ui| {
                    ui.set_min_width(230.0);
                    if ui.button("Create a Bottle").clicked() {
                        state.create_dialog = true;
                        state.show_add_menu = false;
                    }
                    if ui.button("Install a Game with UMU").clicked() {
                        state.navigate(View::Umu);
                        state.show_add_menu = false;
                    }
                });
        }
        if state.show_main_menu {
            egui::Window::new("Main Menu")
                .id(egui::Id::new("main-menu"))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::RIGHT_TOP, vec2(-18.0, 54.0))
                .frame(theme::card_frame(palette, 12))
                .show(ctx, |ui| {
                    ui.set_min_width(220.0);
                    if ui.button("Import…").clicked() {
                        state.navigate(View::Importer);
                        state.show_main_menu = false;
                    }
                    if ui.button("Journal").clicked() {
                        state.navigate(View::Journal);
                        state.show_main_menu = false;
                    }
                    ui.separator();
                    if ui.button("Preferences").clicked() {
                        state.navigate(View::Preferences);
                        state.show_main_menu = false;
                    }
                    if ui.button("Keyboard Shortcuts").clicked() {
                        state.navigate(View::Help);
                        state.show_main_menu = false;
                    }
                    if ui.button("Help").clicked() {
                        state.help_dialog = true;
                        state.show_main_menu = false;
                    }
                    if ui.button("About Bottles").clicked() {
                        state.navigate(View::About);
                        state.show_main_menu = false;
                    }
                });
        }
        if state.show_power_menu {
            egui::Window::new("Power Options")
                .id(egui::Id::new("power-menu"))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::RIGHT_TOP, vec2(-110.0, 54.0))
                .frame(theme::card_frame(palette, 12))
                .show(ctx, |ui| {
                    ui.set_min_width(260.0);
                    if ui.button("Force Stop all Processes").clicked() {
                        state.run_wine_tool(WineTool::Wineserver, &["-k"]);
                        state.show_power_menu = false;
                    }
                    if ui.button("Shutdown").clicked() {
                        state.run_wine_tool(WineTool::Wineboot, &["-s"]);
                        state.show_power_menu = false;
                    }
                    if ui.button("Reboot").clicked() {
                        state.run_wine_tool(WineTool::Wineboot, &["-r"]);
                        state.show_power_menu = false;
                    }
                });
        }
        if state.show_context_menu {
            egui::Window::new("Bottle Menu")
                .id(egui::Id::new("context-menu"))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::RIGHT_TOP, vec2(-70.0, 54.0))
                .frame(theme::card_frame(palette, 12))
                .show(ctx, |ui| {
                    ui.set_min_width(250.0);
                    if ui.button("Duplicate Bottle…").clicked() {
                        state.duplicate_dialog = true;
                        state.show_context_menu = false;
                    }
                    if ui.button("Rename Bottle…").clicked() {
                        state.rename_dialog = true;
                        state.show_context_menu = false;
                    }
                    if ui.button("Full Backup…").clicked() {
                        state.backup_dialog = true;
                        state.show_context_menu = false;
                    }
                    if ui.button("Export Configuration…").clicked() {
                        state.backup_dialog = true;
                        state.export_configuration();
                        state.show_context_menu = false;
                    }
                    ui.separator();
                    if ui
                        .button(if state.show_hidden_programs {
                            "Hide Hidden Programs"
                        } else {
                            "Show Hidden Programs"
                        })
                        .clicked()
                    {
                        state.show_hidden_programs = !state.show_hidden_programs;
                        state.show_context_menu = false;
                    }
                    if ui.button("Delete Bottle…").clicked() {
                        state.delete_dialog = true;
                        state.show_context_menu = false;
                    }
                });
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context, palette: &Palette) {
        let state = &mut self.state;
        if state.create_dialog {
            egui::Window::new("Create New Bottle")
                .id(egui::Id::new("create-dialog"))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .min_width(500.0)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .frame(theme::card_frame(palette, 16))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if theme::secondary_button(ui, palette, "Cancel").clicked() {
                            state.create_dialog = false;
                        }
                        ui.with_layout(
                            Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.label(RichText::new("Create New Bottle").strong());
                            },
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if theme::pill_button(ui, palette, "Create", 90.0).clicked() {
                                state.create_bottle();
                            }
                        });
                    });
                    ui.separator();
                    ui.add_space(6.0);
                    labeled_text(ui, "Name", &mut state.new_bottle_name);
                    ui.add_space(8.0);
                    ui.label(RichText::new("Environment").strong());
                    theme::card(ui, palette, |ui| {
                        environment_option(
                            ui,
                            palette,
                            "application",
                            "Application",
                            "Optimized for productivity software",
                            &mut state.new_bottle_environment,
                        );
                        environment_option(
                            ui,
                            palette,
                            "gaming",
                            "Gaming",
                            "Optimized for games, game engines, and 3D apps",
                            &mut state.new_bottle_environment,
                        );
                        environment_option(
                            ui,
                            palette,
                            "custom",
                            "Custom",
                            "A clean state intended for specific use cases",
                            &mut state.new_bottle_environment,
                        );
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new("Custom").strong());
                    theme::card(ui, palette, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Architecture");
                            egui::ComboBox::from_id_salt("new-arch")
                                .selected_text(if state.new_bottle_arch == "win64" {
                                    "64-bit"
                                } else {
                                    "32-bit"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut state.new_bottle_arch,
                                        "win64".to_owned(),
                                        "64-bit",
                                    );
                                    ui.selectable_value(
                                        &mut state.new_bottle_arch,
                                        "win32".to_owned(),
                                        "32-bit",
                                    );
                                });
                        });
                        let runners = catalogs().component_names("runners");
                        combo_from_list(ui, "Runner", &runners, &mut state.new_bottle_runner);
                        labeled_text(ui, "Bottle Directory", &mut state.new_bottle_custom_path);
                        ui.checkbox(&mut state.new_bottle_sandbox, "Dedicated Sandbox");
                    });
                });
        }

        simple_dialog(
            ctx,
            palette,
            "Run Executable",
            "run-dialog",
            Dialog::Run,
            state,
            |ui, state| {
                labeled_text(ui, "Executable path", &mut state.run_executable);
                labeled_text(ui, "Arguments", &mut state.exec_arguments);
                ui.checkbox(&mut state.exec_terminal, "Run in terminal");
                ui.checkbox(&mut state.exec_winebridge, "Run using Winebridge");
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Run", 100.0).clicked() {
                    state.launch_selected_executable();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Add Shortcuts",
            "shortcut-dialog",
            Dialog::Shortcut,
            state,
            |ui, state| {
                labeled_text(ui, "Name", &mut state.program_name);
                labeled_text(ui, "Path", &mut state.program_path);
                labeled_text(ui, "Arguments", &mut state.program_arguments);
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Add", 100.0).clicked() {
                    state.add_selected_program();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Rename Bottle",
            "rename-dialog",
            Dialog::Rename,
            state,
            |ui, state| {
                labeled_text(ui, "Name", &mut state.rename_input);
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Rename", 100.0).clicked() {
                    state.rename_selected();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Duplicate Bottle",
            "duplicate-dialog",
            Dialog::Duplicate,
            state,
            |ui, state| {
                labeled_text(ui, "Name", &mut state.duplicate_input);
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Duplicate", 110.0).clicked() {
                    state.duplicate_selected();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Delete Bottle",
            "delete-dialog",
            Dialog::Delete,
            state,
            |ui, state| {
                ui.label("Deleting a bottle removes its files permanently.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, palette, "Cancel").clicked() {
                        state.delete_dialog = false;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Delete").color(Color32::WHITE))
                                .fill(palette.danger)
                                .corner_radius(CornerRadius::same(8)),
                        )
                        .clicked()
                    {
                        state.delete_selected();
                    }
                });
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Full Backup",
            "backup-dialog",
            Dialog::Backup,
            state,
            |ui, state| {
                labeled_text(ui, "Destination", &mut state.backup_path);
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Create Backup", 140.0).clicked() {
                    state.full_backup();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Launch Options",
            "launch-options-dialog",
            Dialog::LaunchOptions,
            state,
            |ui, state| {
                labeled_text(ui, "Arguments", &mut state.exec_arguments);
                ui.checkbox(&mut state.exec_terminal, "Run in Terminal");
                ui.checkbox(&mut state.exec_winebridge, "Run using Winebridge");
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Environment Variables",
            "env-dialog",
            Dialog::Environment,
            state,
            |ui, state| {
                labeled_text(ui, "Name", &mut state.env_key);
                labeled_text(ui, "Value", &mut state.env_value);
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Add", 100.0).clicked() {
                    state.add_environment_variable();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "New Registry Rule",
            "registry-dialog",
            Dialog::Registry,
            state,
            |ui, state| {
                labeled_text(ui, "Name", &mut state.rule_name);
                labeled_text(ui, "Description", &mut state.rule_description);
                ui.label("Registry keys");
                ui.add(
                    egui::TextEdit::multiline(&mut state.rule_keys)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6),
                );
                ui.add_space(8.0);
                if theme::pill_button(ui, palette, "Save Rule", 110.0).clicked() {
                    state.add_registry_rule();
                }
            },
        );

        simple_dialog(
            ctx,
            palette,
            "Help",
            "help-dialog",
            Dialog::Help,
            state,
            |ui, _state| {
                ui.label("Bottles documentation is available at:");
                ui.hyperlink(format!("https://{DOC_URL}"));
                ui.add_space(8.0);
                ui.label("Press Esc to close dialogs or go back.");
            },
        );

        simple_dialog(
            ctx,
            palette,
            "About Bottles",
            "about-dialog",
            Dialog::About,
            state,
            |ui, _state| {
                ui.label(RichText::new(APP_NAME).strong().size(20.0));
                ui.label(format!("Version {APP_VERSION}"));
                ui.label(APP_ID);
            },
        );
    }
}

fn field(entry: &Value, key: &str) -> Option<String> {
    entry
        .as_mapping()?
        .get(&Value::String(key.to_owned()))?
        .as_str()
        .map(str::to_owned)
}

fn nice_arch(arch: &str) -> String {
    match arch {
        "win64" => "Win64".to_owned(),
        "win32" => "Win32".to_owned(),
        _ => arch.to_owned(),
    }
}

fn cover_color(name: &str, palette: &Palette) -> Color32 {
    let sum: u32 = name.bytes().map(u32::from).sum();
    let hue = (sum % 360) as f32;
    let (red, green, blue) = hsv_to_rgb(hue, 0.48, if palette.dark { 0.46 } else { 0.68 });
    Color32::from_rgb(red, green, blue)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let chroma = value * saturation;
    let hue = hue % 360.0;
    let x = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = value - chroma;
    let (r, g, b) = match (hue / 60.0) as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

fn draw_bottle_logo(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    let width = rect.width() / 4.2;
    for index in 0..3 {
        let center_x = rect.left() + rect.width() * (index as f32 + 0.5) / 3.0;
        let bottom = rect.bottom();
        let top = rect.top() + rect.height() * 0.12;
        let shoulder = rect.top() + rect.height() * 0.35;
        let neck = width * 0.42;
        let body = width * 0.62;
        let points = vec![
            pos2(center_x - neck, top),
            pos2(center_x + neck, top),
            pos2(center_x + neck, shoulder - 2.0),
            pos2(center_x + body, shoulder),
            pos2(center_x + body, bottom),
            pos2(center_x - body, bottom),
            pos2(center_x - body, shoulder),
            pos2(center_x - neck, shoulder - 2.0),
        ];
        painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
    }
}

fn labeled_text(ui: &mut Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
    });
}

fn environment_option(
    ui: &mut Ui,
    palette: &Palette,
    value: &str,
    title: &str,
    subtitle: &str,
    selected: &mut String,
) {
    ui.horizontal(|ui| {
        ui.radio_value(selected, value.to_owned(), "");
        ui.vertical(|ui| {
            ui.label(RichText::new(title).strong());
            ui.label(RichText::new(subtitle).color(palette.subtext).size(11.5));
        });
    });
    ui.separator();
}

fn component_combo(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    subtitle: &str,
    category: &str,
    selected: &mut String,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).strong());
            ui.label(RichText::new(subtitle).color(palette.subtext).size(11.5));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let names = catalogs().component_names(category);
            let mut current = selected.clone();
            if current.is_empty() {
                current = names.first().cloned().unwrap_or_default();
                *selected = current.clone();
            }
            egui::ComboBox::from_id_salt(format!("component-{category}-{title}"))
                .selected_text(&current)
                .width(180.0)
                .show_ui(ui, |ui| {
                    for name in names {
                        ui.selectable_value(selected, name.clone(), name);
                    }
                });
        });
    });
    ui.separator();
}

fn combo_from_list(ui: &mut Ui, label: &str, items: &[String], selected: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if selected.is_empty() {
                if let Some(first) = items.first() {
                    *selected = first.clone();
                }
            }
            egui::ComboBox::from_id_salt(format!("combo-{label}"))
                .selected_text(selected.as_str())
                .width(220.0)
                .show_ui(ui, |ui| {
                    for item in items {
                        ui.selectable_value(selected, item.clone(), item);
                    }
                });
        });
    });
}

fn setting_switch(ui: &mut Ui, palette: &Palette, title: &str, subtitle: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).strong());
            ui.label(RichText::new(subtitle).color(palette.subtext).size(11.5));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            theme::switch(ui, palette, value);
        });
    });
    ui.separator();
}

fn bottle_context_menu(ui: &mut Ui, state: &mut AppState, name: &str) {
    let name = name.to_owned();
    if ui.button("Rename Bottle…").clicked() {
        state.select_bottle(name.clone());
        state.rename_dialog = true;
    }
    if ui.button("Duplicate Bottle…").clicked() {
        state.select_bottle(name.clone());
        state.duplicate_dialog = true;
    }
    if ui.button("Delete Bottle…").clicked() {
        state.select_bottle(name.clone());
        state.delete_dialog = true;
    }
}

fn simple_dialog(
    ctx: &egui::Context,
    palette: &Palette,
    title: &str,
    id: &str,
    dialog: Dialog,
    state: &mut AppState,
    contents: impl FnOnce(&mut Ui, &mut AppState),
) {
    if !state.dialog_open(dialog) {
        return;
    }
    egui::Window::new(title)
        .id(egui::Id::new(id))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .min_width(460.0)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .frame(theme::card_frame(palette, 16))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::icon_button(ui, palette, Icon::Close, "Close").clicked() {
                        state.close_dialog(dialog);
                    }
                });
            });
            ui.separator();
            ui.add_space(6.0);
            contents(ui, state);
        });
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

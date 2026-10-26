# Bottles Rust/egui parity report

This report is generated from the existing Python application by
`rust/scripts/generate_contracts.py`. It is not a hand-written wish list.

## Status meanings

- `ported`: the Rust code implements the behavior needed for this tracked surface.
- `partial`: command/model/data surface exists, but Python behavior is broader.
- `inventory-only`: the UI surface is cataloged and visible in the egui Parity screen, but has no dedicated Rust dialog yet.
- `reference-only`: the Python source is the sole implementation; use the Python application for this subsystem.

## Source-derived inventory

| Contract | Count | Source |
|---|---:|---|
| BottleConfig fields | 35 | `bottles/backend/models/config.py` |
| BottleParams fields | 56 | `bottles/backend/models/config.py` |
| BottleSandboxParams fields | 4 | `bottles/backend/models/config.py` |
| GSettings keys | 33 | `data/com.usebottles.bottles.gschema.xml` |
| CLI commands | 15 | `bottles/frontend/cli/cli.py` |
| Blueprint UI surfaces | 78 | `bottles/frontend/ui` |
| Python modules | 208 | `bottles` |

## Status totals

| Status | Count |
|---|---:|
| `inventory-only` | 56 |
| `partial` | 12 |
| `ported` | 39 |
| `reference-only` | 194 |

## Implemented compatibility layer

- Exact `bottle.yml` model, defaults, legacy migrations, and atomic writes.
- XDG/Flatpak path discovery and the same application data directories.
- Bottle create/list/rename/duplicate/delete, custom-path placeholders, and prefix import.
- External programs, environment variables, DLL overrides, registry rules, and component metadata.
- `library.yml`, `journal.yml`, `data.yml`, GSettings schema, and UMU `game.yml` compatibility.
- Wine runner resolution, environment construction, file-type dispatch, Wine tools, registry operations, standalone scripts, and process launch.
- The full Python CLI command surface with JSON/text output.
- An egui/eframe front end modeled on the checked-in GTK4/Blueprint screens: header-bar view switcher, bottle list, library cards, bottle details, settings, dependencies, installers, snapshots, task manager, registry rules, preferences, dialogs, and source-derived diagnostics.
- Unit tests that compare Rust field/command/settings lists against generated Python contracts.

## CLI port status

| Command | Status |
|---|---|
| `info` | `ported` |
| `list` | `ported` |
| `programs` | `ported` |
| `add` | `ported` |
| `tools` | `ported` |
| `reg` | `ported` |
| `reg-rules` | `ported` |
| `edit` | `ported` |
| `new` | `ported` |
| `run` | `ported` |
| `stop` | `ported` |
| `standalone` | `ported` |
| `shell` | `ported` |
| `umu` | `partial` |
| `autostart` | `ported` |

## Blueprint UI port status

| Blueprint | Kind | Status | Rust surface |
|---|---|---|---|
| `bottle-row.blp` | screen | `inventory-only` | — |
| `bulk-update-dialog.blp` | screen | `inventory-only` | — |
| `check-row.blp` | screen | `inventory-only` | — |
| `component-entry.blp` | screen | `inventory-only` | — |
| `dependency-entry.blp` | screen | `inventory-only` | — |
| `details-bottle.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `details-dependencies.blp` | screen | `partial` | `crates/bottles-gui/src/ui.rs` |
| `details-installers.blp` | screen | `partial` | `crates/bottles-gui/src/ui.rs` |
| `details-preferences.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `details-registry-rules.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `details-taskmanager.blp` | screen | `partial` | `crates/bottles-gui/src/ui.rs` |
| `details-versioning.blp` | screen | `partial` | `crates/bottles-gui/src/ui.rs` |
| `details.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-bottle-picker.blp` | dialog | `inventory-only` | — |
| `dialog-crash-report.blp` | dialog | `inventory-only` | — |
| `dialog-dependency-install.blp` | dialog | `inventory-only` | — |
| `dialog-deps-check.blp` | dialog | `inventory-only` | — |
| `dialog-display.blp` | dialog | `inventory-only` | — |
| `dialog-dll-overrides.blp` | dialog | `inventory-only` | — |
| `dialog-drives.blp` | dialog | `inventory-only` | — |
| `dialog-duplicate.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-env-vars.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-exclusion-patterns.blp` | dialog | `inventory-only` | — |
| `dialog-file-associations.blp` | dialog | `inventory-only` | — |
| `dialog-gamescope.blp` | dialog | `inventory-only` | — |
| `dialog-installer.blp` | dialog | `inventory-only` | — |
| `dialog-journal.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-launch-options.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-lsfg-vk.blp` | dialog | `inventory-only` | — |
| `dialog-mangohud.blp` | dialog | `inventory-only` | — |
| `dialog-playtime-graph.blp` | dialog | `inventory-only` | — |
| `dialog-program-backups.blp` | dialog | `inventory-only` | — |
| `dialog-proton-alert.blp` | dialog | `inventory-only` | — |
| `dialog-registry-rules.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-rename.blp` | dialog | `ported` | `crates/bottles-gui/src/ui.rs` |
| `dialog-run-args.blp` | dialog | `inventory-only` | — |
| `dialog-sandbox.blp` | dialog | `inventory-only` | — |
| `dialog-umu-add.blp` | dialog | `inventory-only` | — |
| `dialog-umu-arguments.blp` | dialog | `inventory-only` | — |
| `dialog-umu-dependencies.blp` | dialog | `inventory-only` | — |
| `dialog-umu-environment.blp` | dialog | `inventory-only` | — |
| `dialog-umu-game.blp` | dialog | `inventory-only` | — |
| `dialog-umu-install.blp` | dialog | `inventory-only` | — |
| `dialog-umu-proton.blp` | dialog | `inventory-only` | — |
| `dialog-umu-search.blp` | dialog | `inventory-only` | — |
| `dialog-upgrade-versioning.blp` | dialog | `inventory-only` | — |
| `dialog-versioning-branch.blp` | dialog | `inventory-only` | — |
| `dialog-versioning-commit.blp` | dialog | `inventory-only` | — |
| `dialog-versioning-manage-branches.blp` | dialog | `inventory-only` | — |
| `dialog-versioning-settings.blp` | dialog | `inventory-only` | — |
| `dialog-vkbasalt.blp` | dialog | `inventory-only` | — |
| `dialog-vmtouch.blp` | dialog | `inventory-only` | — |
| `dialog-winebridge-update.blp` | dialog | `inventory-only` | — |
| `dll-override-entry.blp` | screen | `inventory-only` | — |
| `drive-entry.blp` | screen | `inventory-only` | — |
| `eagle.blp` | screen | `inventory-only` | — |
| `env-var-entry.blp` | screen | `inventory-only` | — |
| `exclusion-pattern-entry.blp` | screen | `inventory-only` | — |
| `help-overlay.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `importer-entry.blp` | screen | `inventory-only` | — |
| `importer.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `inherited-env-entry.blp` | screen | `inventory-only` | — |
| `installer-entry.blp` | screen | `inventory-only` | — |
| `library-add-entry.blp` | screen | `inventory-only` | — |
| `library-entry.blp` | screen | `inventory-only` | — |
| `library.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `list.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `loading.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `local-resource-entry.blp` | screen | `inventory-only` | — |
| `new-bottle-dialog.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `onboard.blp` | screen | `inventory-only` | — |
| `preferences.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |
| `program-entry.blp` | screen | `inventory-only` | — |
| `registry-rule-entry.blp` | screen | `inventory-only` | — |
| `state-entry.blp` | screen | `inventory-only` | — |
| `task-entry.blp` | screen | `inventory-only` | — |
| `umu-prefix-row.blp` | screen | `inventory-only` | — |
| `window.blp` | screen | `ported` | `crates/bottles-gui/src/ui.rs` |

## Python modules still serving as reference

These modules have no direct Rust counterpart. They are all listed in the
egui Parity screen so no source file is silently omitted.

### `backend` (113)

- `bottles/backend/cabextract.py`
- `bottles/backend/diff.py`
- `bottles/backend/dlls/d7vk.py`
- `bottles/backend/dlls/dll.py`
- `bottles/backend/dlls/dxvk.py`
- `bottles/backend/dlls/latencyflex.py`
- `bottles/backend/dlls/nvapi.py`
- `bottles/backend/dlls/vkd3d.py`
- `bottles/backend/downloader.py`
- `bottles/backend/identity.py`
- `bottles/backend/identity_ui.py`
- `bottles/backend/logger.py`
- `bottles/backend/managers/backup.py`
- `bottles/backend/managers/component.py`
- `bottles/backend/managers/conf.py`
- `bottles/backend/managers/dependency.py`
- `bottles/backend/managers/eagle.py`
- `bottles/backend/managers/eaglenative.py`
- `bottles/backend/managers/eagletracing.py`
- `bottles/backend/managers/epicgamesstore.py`
- `bottles/backend/managers/installer.py`
- `bottles/backend/managers/intel.py`
- `bottles/backend/managers/origin.py`
- `bottles/backend/managers/playtime.py`
- `bottles/backend/managers/queue.py`
- `bottles/backend/managers/repository.py`
- `bottles/backend/managers/runtime.py`
- `bottles/backend/managers/sandbox.py`
- `bottles/backend/managers/steam.py`
- `bottles/backend/managers/steamgriddb.py`
- `bottles/backend/managers/template.py`
- `bottles/backend/managers/thumbnail.py`
- `bottles/backend/managers/ubisoftconnect.py`
- `bottles/backend/managers/versioning.py`
- `bottles/backend/models/enum.py`
- `bottles/backend/models/process.py`
- `bottles/backend/models/result.py`
- `bottles/backend/models/samples.py`
- `bottles/backend/models/vdict.py`
- `bottles/backend/params.py`
- `bottles/backend/repos/component.py`
- `bottles/backend/repos/dependency.py`
- `bottles/backend/repos/installer.py`
- `bottles/backend/repos/repo.py`
- `bottles/backend/runner.py`
- `bottles/backend/umu/database.py`
- `bottles/backend/umu/dependencies.py`
- `bottles/backend/umu/executor.py`
- `bottles/backend/umu/models.py`
- `bottles/backend/umu/processes.py`
- `bottles/backend/umu/proton.py`
- `bottles/backend/umu/provider.py`
- `bottles/backend/umu/repository.py`
- `bottles/backend/utils/connection.py`
- `bottles/backend/utils/decorators.py`
- `bottles/backend/utils/display.py`
- `bottles/backend/utils/file.py`
- `bottles/backend/utils/generic.py`
- `bottles/backend/utils/gpu.py`
- `bottles/backend/utils/gsettings_stub.py`
- `bottles/backend/utils/hidraw.py`
- `bottles/backend/utils/imagemagick.py`
- `bottles/backend/utils/json.py`
- `bottles/backend/utils/lnk.py`
- `bottles/backend/utils/lsfgvk.py`
- `bottles/backend/utils/manager.py`
- `bottles/backend/utils/nvidia.py`
- `bottles/backend/utils/proc.py`
- `bottles/backend/utils/singleton.py`
- `bottles/backend/utils/snake.py`
- `bottles/backend/utils/steam.py`
- `bottles/backend/utils/terminal.py`
- `bottles/backend/utils/threading.py`
- `bottles/backend/utils/vdf.py`
- `bottles/backend/utils/vulkan.py`
- `bottles/backend/utils/wine.py`
- `bottles/backend/utils/yaml.py`
- `bottles/backend/wine/adaptive.py`
- `bottles/backend/wine/catalogs.py`
- `bottles/backend/wine/cmd.py`
- `bottles/backend/wine/control.py`
- `bottles/backend/wine/drives.py`
- `bottles/backend/wine/eject.py`
- `bottles/backend/wine/executor.py`
- `bottles/backend/wine/expand.py`
- `bottles/backend/wine/explorer.py`
- `bottles/backend/wine/hh.py`
- `bottles/backend/wine/icinfo.py`
- `bottles/backend/wine/msiexec.py`
- `bottles/backend/wine/net.py`
- `bottles/backend/wine/notepad.py`
- `bottles/backend/wine/oleview.py`
- `bottles/backend/wine/progman.py`
- `bottles/backend/wine/reg.py`
- `bottles/backend/wine/regedit.py`
- `bottles/backend/wine/register.py`
- `bottles/backend/wine/regkeys.py`
- `bottles/backend/wine/regsvr32.py`
- `bottles/backend/wine/rundll32.py`
- `bottles/backend/wine/start.py`
- `bottles/backend/wine/taskmgr.py`
- `bottles/backend/wine/uninstaller.py`
- `bottles/backend/wine/wineboot.py`
- `bottles/backend/wine/winebridge.py`
- `bottles/backend/wine/winecfg.py`
- `bottles/backend/wine/winecommand.py`
- `bottles/backend/wine/winedbg.py`
- `bottles/backend/wine/winefile.py`
- `bottles/backend/wine/winepath.py`
- `bottles/backend/wine/wineprogram.py`
- `bottles/backend/wine/wineserver.py`
- `bottles/backend/wine/winhelp.py`
- `bottles/backend/wine/xcopy.py`

### `frontend` (79)

- `bottles/frontend/bottles.py`
- `bottles/frontend/cli/utils.py`
- `bottles/frontend/operation.py`
- `bottles/frontend/params.py`
- `bottles/frontend/utils/autostart.py`
- `bottles/frontend/utils/common.py`
- `bottles/frontend/utils/filters.py`
- `bottles/frontend/utils/flatpak.py`
- `bottles/frontend/utils/gtk.py`
- `bottles/frontend/utils/localization.py`
- `bottles/frontend/utils/playtime.py`
- `bottles/frontend/utils/sandbox_guard.py`
- `bottles/frontend/utils/sh.py`
- `bottles/frontend/utils/umu.py`
- `bottles/frontend/views/bottle_dependencies.py`
- `bottles/frontend/views/bottle_details.py`
- `bottles/frontend/views/bottle_installers.py`
- `bottles/frontend/views/bottle_preferences.py`
- `bottles/frontend/views/bottle_registry_rules.py`
- `bottles/frontend/views/bottle_taskmanager.py`
- `bottles/frontend/views/bottle_versioning.py`
- `bottles/frontend/views/details.py`
- `bottles/frontend/views/eagle.py`
- `bottles/frontend/views/importer.py`
- `bottles/frontend/views/library.py`
- `bottles/frontend/views/list.py`
- `bottles/frontend/views/loading.py`
- `bottles/frontend/views/new_bottle_dialog.py`
- `bottles/frontend/views/preferences.py`
- `bottles/frontend/widgets/component.py`
- `bottles/frontend/widgets/dependency.py`
- `bottles/frontend/widgets/executable.py`
- `bottles/frontend/widgets/importer.py`
- `bottles/frontend/widgets/installer.py`
- `bottles/frontend/widgets/library.py`
- `bottles/frontend/widgets/playtimechart_hourly.py`
- `bottles/frontend/widgets/playtimechart_monthly.py`
- `bottles/frontend/widgets/playtimechart_weekly.py`
- `bottles/frontend/widgets/program.py`
- `bottles/frontend/widgets/state.py`
- `bottles/frontend/widgets/umu.py`
- `bottles/frontend/windows/bottlepicker.py`
- `bottles/frontend/windows/bulkupdate.py`
- `bottles/frontend/windows/crash.py`
- `bottles/frontend/windows/dependency_install.py`
- `bottles/frontend/windows/depscheck.py`
- `bottles/frontend/windows/display.py`
- `bottles/frontend/windows/dlloverrides.py`
- `bottles/frontend/windows/drives.py`
- `bottles/frontend/windows/duplicate.py`
- `bottles/frontend/windows/eagleintel.py`
- `bottles/frontend/windows/envvars.py`
- `bottles/frontend/windows/exclusionpatterns.py`
- `bottles/frontend/windows/fileassociations.py`
- `bottles/frontend/windows/funding.py`
- `bottles/frontend/windows/gamescope.py`
- `bottles/frontend/windows/generic.py`
- `bottles/frontend/windows/generic_cli.py`
- `bottles/frontend/windows/installer.py`
- `bottles/frontend/windows/journal.py`
- `bottles/frontend/windows/launchoptions.py`
- `bottles/frontend/windows/lsfgvk.py`
- `bottles/frontend/windows/mangohud.py`
- `bottles/frontend/windows/onboard.py`
- `bottles/frontend/windows/playtimegraph.py`
- `bottles/frontend/windows/programbackups.py`
- `bottles/frontend/windows/protonalert.py`
- `bottles/frontend/windows/registry_rules.py`
- `bottles/frontend/windows/rename.py`
- `bottles/frontend/windows/sandbox.py`
- `bottles/frontend/windows/umu.py`
- `bottles/frontend/windows/upgradeversioning.py`
- `bottles/frontend/windows/versioning_branch.py`
- `bottles/frontend/windows/versioning_commit.py`
- `bottles/frontend/windows/versioning_manage_branches.py`
- `bottles/frontend/windows/versioning_settings.py`
- `bottles/frontend/windows/vkbasalt.py`
- `bottles/frontend/windows/vmtouch.py`
- `bottles/frontend/windows/winebridgeupdate.py`

### `fvs` (2)

- `bottles/fvs/exceptions.py`
- `bottles/fvs/repo.py`

## Honest boundary

The Python codebase contains approximately 100k lines including tests.
A full behavioral port of network repositories, component downloads,
dependency/installer recipes, Steam/Epic/Ubisoft integrations, Eagle/YARA
security analysis, FVS versioning, playtime SQLite, sandboxing, Proton/UMU
runtime management, and every GTK dialog is a multi-stage project. The Rust
workspace replaces the application shell and compatibility core without
pretending that those subsystems are complete. The Python tree remains in
the repository as the executable specification and fallback implementation.

# Rust/egui rework plan

This plan is the contract for rebuilding the Rust GUI. It is derived from the
GTK4/libadwaita sources and screenshots already checked into this repository,
not from an imagined application. The previous generic dashboard-style rewrite
was rejected; this document exists so the UI structure is fixed before Rust
widgets are written.

## Goal

Build an `egui`/`eframe` front end that is recognisably Bottles:

- the same two primary views, **Bottles** and **Library**;
- the same header-bar navigation and menu structure;
- the same bottle list, library card grid, and bottle details hierarchy;
- the same labels, descriptions, grouping, and visual language;
- source-derived installer, dependency, and component catalogs;
- real actions backed by the Rust compatibility core, with explicit messages
  where a Python subsystem is still authoritative.

The Python tree remains the behavioural reference and the fallback for
subsystems that are not ported. The plan does **not** claim parity with network
installer execution, dependency recipes, FVS snapshots, Eagle/YARA scanning,
playtime SQLite, sandboxing, or every GTK dialog.

## Reference material in this repository

| Area | Source |
|---|---|
| Window shell, view switcher, add popover, main menu | `bottles/frontend/ui/window.blp` |
| Bottle list, search, update banner, empty state | `bottles/frontend/ui/list.blp` |
| Library flow box and empty state | `bottles/frontend/ui/library.blp` |
| Details leaflet and subpage stack | `bottles/frontend/ui/details.blp` |
| Bottle overview, programs, options, tools, context/power menus | `bottles/frontend/ui/details-bottle.blp` |
| Settings groups and exact setting labels | `bottles/frontend/ui/details-preferences.blp` |
| Dependencies page | `bottles/frontend/ui/details-dependencies.blp` |
| Installers page | `bottles/frontend/ui/details-installers.blp` |
| Snapshots/versioning page | `bottles/frontend/ui/details-versioning.blp` |
| Task manager page | `bottles/frontend/ui/details-taskmanager.blp` |
| Registry rules page | `bottles/frontend/ui/details-registry-rules.blp` |
| Create-bottle dialog | `bottles/frontend/ui/new-bottle-dialog.blp` |
| Preferences window | `bottles/frontend/ui/preferences.blp` |
| Importer | `bottles/frontend/ui/importer.blp` |
| Colours and custom widget styling | `bottles/frontend/ui/style.css`, `bottles/frontend/ui/style-dark.css` |
| Symbolic icons | `data/icons/hicolor/symbolic/actions/`, `data/icons/hicolor/symbolic/apps/` |
| Visual reference screenshots | `data/screenshots/1.png` … `data/screenshots/6.png` |
| Pinned upstream repository commits | `bottles/backend/managers/repository.py` |

## Visual system

The egui theme must reproduce the Adwaita-like presentation shown in the
checked-in screenshots:

- light and dark palettes selected by the existing `dark-theme` setting;
- accent blue for suggested actions and switches, approximately `#3584e4`;
- a 48 px header bar with a subtle bottom border;
- a centred segmented view switcher for **Bottles** / **Library**;
- left header action: `+` add popover;
- right header actions: support heart, search, main menu, close;
- detail pages replace the centre switcher with a title and a back button;
- rounded `boxed-list` cards with 12 px corners, 1 px borders, and 12 px
  padding;
- 44 px single-line rows and 58 px two-line `ActionRow` equivalents;
- pill-shaped badges for architecture, environment, runner, and installer
  grades;
- symbolic-looking icons drawn as vector primitives rather than emoji;
- suggested pill buttons for primary actions;
- secondary buttons with borders and no strong fill;
- muted subtitles and section titles above grouped cards;
- status pages with a large logo, title, description, and one primary action.

No left navigation sidebar, generic search bar, parity inventory in the main
shell, or admin-dashboard chrome.

## Navigation model

### Root

`Adw.ViewStack` equivalent:

1. **Bottles** — bottle list, UMU prefixes, Steam Proton prefixes.
2. **Library** — a wrapping flow box of library cards.

Search is a collapsible search bar below the header, only for these two views.

### Bottle details leaf

Selecting a bottle row pushes a `Details` leaf:

- back button returns to root;
- header title is **Details**;
- power menu and secondary menu are available;
- power menu: Force Stop all Processes, Force Stop all Processes (Native
  method), Shutdown, Reboot;
- secondary menu: Troubleshooting, Duplicate Bottle, Full Backup, Export
  Configuration, Show Hidden Programs, Search for new programs, Delete Bottle.

The details page itself contains:

- centred bottle name;
- `Win64 · Environment · Runner` tags and optional snapshot badge;
- a large **Run Executable…** pill button with a launch-options button;
- **Programs** card with program rows, **Add Shortcuts…**, and
  **Install Programs…**;
- **Options** card: Settings, Dependencies, Registry Rules, Snapshots,
  Task Manager;
- **Tools** card: Command Line, Registry Editor, Explorer, Task Manager,
  Configuration, Uninstaller, Control Panel, Debugger, Legacy Wine Tools,
  Browse C:/ drive, and Launch Options.

### Details subpages

A stack of subpages reachable from Options:

| Subpage | Source | Required content |
|---|---|---|
| Settings | `details-preferences.blp` | Name, Components, Display, Performance, Compatibility, Advanced; combo rows and switches with exact labels; Save action |
| Dependencies | `details-dependencies.blp` | Search, licensing description, source-derived dependency list, Select All / Install Selected controls |
| Installers | `details-installers.blp` | Search, licensing description, installer rows with grade badge, download and menu actions |
| Registry Rules | `details-registry-rules.blp` | Search, empty state, rule list, New Rule, Apply Rules |
| Snapshots | `details-versioning.blp` | Branch row, state list, no-snapshots status page, Create Snapshot action |
| Task Manager | `details-taskmanager.blp` | Process/task table with Refresh and Stop process actions |

### Global pages

Reachable from the main menu or preferences:

- Import Wine prefixes / backups (`importer.blp`);
- Journal (`dialog-journal.blp`);
- Preferences (`preferences.blp`);
- Keyboard Shortcuts (`help-overlay.blp`);
- Help and About Bottles;
- Components, System Health, UMU, and Parity diagnostics as clearly labelled
  developer pages, not as primary navigation.

### Dialogs and popovers

- Create New Bottle: Cancel / Create header, Name entry, Application / Gaming /
  Custom radio rows, Architecture, Runner, Dedicated Sandbox, Bottle Directory,
  Import Configuration;
- Run Executable: executable path, arguments, Run in Terminal, Run using
  Winebridge;
- Add Shortcuts: name, path, arguments;
- Rename, Duplicate, Delete;
- Full Backup and Export Configuration;
- Environment Variables and New Registry Rule;
- Launch Options popover;
- Help and About.

## Source-derived data

The installer, dependency, and component screens must not invent entries. The
Python application pins these repositories in
`bottles/backend/managers/repository.py`:

- `bottlesdevs/programs` commit `60a3de90d1a00ddc0e02393e64d1721a789f8af6`;
- `bottlesdevs/dependencies` commit `23bd12fd717c506addb0b05859a5f8635036f225`;
- `bottlesdevs/components` commit `fdfdc568c7ec9744b6feee21b7519b38aeaaae6d`.

A generator reads each pinned `index.yml`, stores a compact JSON snapshot, and
records the source repository and commit in the snapshot. Installer filtering
must mirror `InstallerManager.supports_channel` and
`InstallerManager.supports_runner`: architecture, stable/unstable channel, and
runner compatibility.

## Functional mapping

| UI action | Rust core requirement |
|---|---|
| List/create/rename/duplicate/delete bottles | `BottleManager` equivalents |
| Bottle rows and details | `bottle.yml` config model |
| Programs and Run Executable | `WineLauncher` plans and process spawn |
| Add Shortcuts | `BottleManager::add_program` equivalent |
| Settings combo rows and switches | typed `BottleConfig` / `BottleParams` |
| Environment variables and registry rules | config mutation and registry-rule manager |
| Library cards | `library.yml` compatibility layer |
| Journal and preferences | journal and GSettings schema equivalents |
| Importer | Wine prefix discovery and backup/config import |
| Full Backup / Export Configuration | prefix archive and `bottle.yml` export |
| UMU | `game.yml` compatibility layer |
| System health / parity | diagnostics and source-derived contracts |

Actions that are not ported must display an honest message identifying the
Python application as the current implementation, rather than pretending to
succeed.

## Implementation phases

1. **Theme and widgets**: Adwaita-like palette, header bar, cards, rows,
   badges, switches, combo rows, symbolic icon primitives, status pages, and
   modal windows.
2. **Shell**: root view switcher, search bar, main menu, add popover, bottle
   list, library grid, and create-bottle dialog.
3. **Details**: overview, programs, options, tools, power/context menus, and
   all subpages.
4. **Dialogs**: run executable, shortcuts, rename, duplicate, delete, backup,
   environment variables, registry rules, launch options, help, about.
5. **Catalogs**: pinned upstream snapshot generator, offline JSON asset, and
   filtering tests.
6. **Verification**: `cargo fmt`, `cargo clippy`, workspace tests, release
   build, and a GitHub Actions workflow that checks and builds the Rust
   workspace.

Each phase must compile before the next one starts. The UI is only acceptable
when the primary screens match the checked-in screenshots in hierarchy,
labels, spacing, and interaction model.

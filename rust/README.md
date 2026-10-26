# Bottles — Rust/egui rewrite workspace

This directory contains a Rust workspace that replaces the Bottles application
shell and compatibility core with a native [`egui`/`eframe`](https://github.com/emilk/egui)
front end and a Rust CLI. It reads and writes the same user data as the Python
implementation in `../bottles`.

> **Scope:** the Python tree is intentionally kept as the executable behavioral
> specification. A 100k-line production application cannot honestly be declared
> fully ported in one change. `PARITY.md` is generated from the Python sources,
> lists every config field, setting, CLI command, Blueprint surface, and Python
> module, and states exactly what is ported, partial, or reference-only. The
> egui application also exposes that inventory in its **Parity** screen.

## Workspace

| Crate | Purpose |
|---|---|
| `bottles-core` | Paths, config model, bottle manager, library, journal, settings, Wine launcher, registry rules, UMU repository, state bus, health checks, and embedded parity contracts. |
| `bottles-cli` | Rust replacement for `bottles-cli`, with the complete Python CLI command surface. |
| `bottles-gui` | `bottles` desktop application built with egui/eframe. |

Generated contracts live in `contracts/`. They are produced by
`scripts/generate_contracts.py` from the Python AST, GSettings XML, and
Blueprint file names. `scripts/generate_parity_report.py` produces `PARITY.md`.
`scripts/generate_gui_catalogs.py` produces the pinned installer, dependency,
and component snapshot used by the GUI.

The GUI structure is documented in
`../docs/rust-egui-rework-plan.md`, with a source-derived visual mockup in
`../docs/rust-egui-mockup.html`. The egui front end follows the checked-in
GTK4/Blueprint screen hierarchy rather than using a generic dashboard layout.

## Build

```bash
cd rust
cargo build --release
cargo test --workspace
```

Run:

```bash
./target/release/bottles          # egui/eframe GUI
./target/release/bottles-cli --help
```

On a Linux desktop, `eframe` needs the usual X11/Wayland/OpenGL development
libraries. The CI workflow installs them for Debian/Ubuntu, builds both release
binaries with `cargo build --locked --release --workspace`, and uploads a
`bottles-rs-linux-x86_64` artifact containing `bottles`, `bottles-cli`, and a
SHA-256 checksum.

## Data compatibility

The Rust code uses the Python app's paths under
`$XDG_DATA_HOME/bottles` (or `~/.local/share/bottles`):

- `bottles/<name>/bottle.yml`
- `bottles/<name>/placeholder.yml` for custom-path bottles
- `library.yml`
- `journal.yml`
- `data.yml`
- `umu/games/<uuid>/game.yml`
- the `com.usebottles.bottles` GSettings schema

`bottle.yml` is modeled with the exact 35 `BottleConfig` fields, 56
`BottleParams` fields, and 4 `BottleSandboxParams` fields from the Python
dataclasses. Legacy `fsr_level`, `show_component_updates`, `DXVK_NVAPI`, and
`LatencyFlex` migrations are preserved.

## What works now

- Create, list, inspect, rename, duplicate, repair, and delete bottles.
- Custom bottle paths and external Wine-prefix import.
- Program entries and launch argument/placeholder handling.
- Bottle parameters, environment variables, DLL overrides, and registry rules.
- Wine runner discovery, Wine tool launching, registry edits, shell commands,
  stop, and standalone script generation.
- Library, journal, user data, GSettings, and UMU game metadata.
- Full CLI parser/output surface, including JSON and UMU actions.
- an egui front end with the Bottles header-bar view switcher, bottle list,
  library cards, bottle details, settings, dependencies, installers, snapshots,
  task manager, registry rules, preferences, importer, journal, UMU, about, and
  source-derived diagnostics screens.
- offline installer, dependency, and component catalogs generated from the
  exact upstream commits pinned by the Python repository manager.
- Unit and integration tests, including contract-drift assertions.

## What is not claimed as complete

Network component/dependency/installer repositories, Steam/Epic/Ubisoft
library integrations, Eagle/YARA analysis, FVS incremental versioning,
playtime SQLite, Bubblewrap sandboxing, Proton catalog/runtime management,
thumbnail/image pipelines, and the 63 one-off GTK dialogs are cataloged as
partial or reference-only. See `PARITY.md`.

The CLI accepts the full Python command surface and returns explicit errors for
operations whose Python behavior has not been ported. It does not silently
pretend that a feature succeeded.

## Regenerating contracts

```bash
python3 scripts/generate_contracts.py
python3 scripts/generate_parity_report.py
python3 scripts/generate_gui_catalogs.py
python3 scripts/generate_contracts.py --check
python3 scripts/generate_parity_report.py --check
python3 scripts/generate_gui_catalogs.py --check
```

CI runs the same checks.

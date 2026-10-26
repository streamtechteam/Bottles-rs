#!/usr/bin/env python3
"""Generate PARITY.md from the source-derived contracts."""
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / "rust" / "contracts"
OUTPUT = ROOT / "rust" / "PARITY.md"


def load(name: str):
    return json.loads((CONTRACTS / name).read_text(encoding="utf-8"))


def render() -> str:
    config = load("config_fields.json")
    settings = load("settings.json")
    cli = load("cli.json")
    ui = load("ui.json")
    modules = load("modules.json")

    statuses = Counter()
    for command in cli["commands"]:
        statuses[command["status"]] += 1
    for screen in ui["screens"]:
        statuses[screen["status"]] += 1
    for module in modules["modules"]:
        statuses[module["status"]] += 1

    lines = [
        "# Bottles Rust/egui parity report",
        "",
        "This report is generated from the existing Python application by",
        "`rust/scripts/generate_contracts.py`. It is not a hand-written wish list.",
        "",
        "## Status meanings",
        "",
        "- `ported`: the Rust code implements the behavior needed for this tracked surface.",
        "- `partial`: command/model/data surface exists, but Python behavior is broader.",
        "- `inventory-only`: the UI surface is cataloged and visible in the egui Parity screen, but has no dedicated Rust dialog yet.",
        "- `reference-only`: the Python source is the sole implementation; use the Python application for this subsystem.",
        "",
        "## Source-derived inventory",
        "",
        "| Contract | Count | Source |",
        "|---|---:|---|",
        f"| BottleConfig fields | {len(config['bottle_config'])} | `{config['source']}` |",
        f"| BottleParams fields | {len(config['bottle_params'])} | `{config['source']}` |",
        f"| BottleSandboxParams fields | {len(config['bottle_sandbox_params'])} | `{config['source']}` |",
        f"| GSettings keys | {len(settings['keys'])} | `{settings['source']}` |",
        f"| CLI commands | {len(cli['commands'])} | `{cli['source']}` |",
        f"| Blueprint UI surfaces | {len(ui['screens'])} | `{ui['source']}` |",
        f"| Python modules | {len(modules['modules'])} | `{modules['source']}` |",
        "",
        "## Status totals",
        "",
        "| Status | Count |",
        "|---|---:|",
    ]
    for status in sorted(statuses):
        lines.append(f"| `{status}` | {statuses[status]} |")
    lines += [
        "",
        "## Implemented compatibility layer",
        "",
        "- Exact `bottle.yml` model, defaults, legacy migrations, and atomic writes.",
        "- XDG/Flatpak path discovery and the same application data directories.",
        "- Bottle create/list/rename/duplicate/delete, custom-path placeholders, and prefix import.",
        "- External programs, environment variables, DLL overrides, registry rules, and component metadata.",
        "- `library.yml`, `journal.yml`, `data.yml`, GSettings schema, and UMU `game.yml` compatibility.",
        "- Wine runner resolution, environment construction, file-type dispatch, Wine tools, registry operations, standalone scripts, and process launch.",
        "- The full Python CLI command surface with JSON/text output.",
        "- An egui/eframe front end modeled on the checked-in GTK4/Blueprint screens: header-bar view switcher, bottle list, library cards, bottle details, settings, dependencies, installers, snapshots, task manager, registry rules, preferences, dialogs, and source-derived diagnostics.",
        "- Unit tests that compare Rust field/command/settings lists against generated Python contracts.",
        "",
        "## CLI port status",
        "",
        "| Command | Status |",
        "|---|---|",
    ]
    for command in cli["commands"]:
        lines.append(f"| `{command['name']}` | `{command['status']}` |")
    lines += [
        "",
        "## Blueprint UI port status",
        "",
        "| Blueprint | Kind | Status | Rust surface |",
        "|---|---|---|---|",
    ]
    for screen in ui["screens"]:
        rust = f"`{screen['rust_surface']}`" if screen["rust_surface"] else "—"
        lines.append(
            f"| `{screen['name']}.blp` | {screen['kind']} | `{screen['status']}` | {rust} |"
        )
    lines += [
        "",
        "## Python modules still serving as reference",
        "",
        "These modules have no direct Rust counterpart. They are all listed in the",
        "egui Parity screen so no source file is silently omitted.",
        "",
    ]
    by_area: dict[str, list[str]] = {}
    for module in modules["modules"]:
        if module["status"] == "reference-only":
            by_area.setdefault(module["area"], []).append(module["source"])
    for area in sorted(by_area):
        lines.append(f"### `{area}` ({len(by_area[area])})")
        lines.append("")
        for source in by_area[area]:
            lines.append(f"- `{source}`")
        lines.append("")

    lines += [
        "## Honest boundary",
        "",
        "The Python codebase contains approximately 100k lines including tests.",
        "A full behavioral port of network repositories, component downloads,",
        "dependency/installer recipes, Steam/Epic/Ubisoft integrations, Eagle/YARA",
        "security analysis, FVS versioning, playtime SQLite, sandboxing, Proton/UMU",
        "runtime management, and every GTK dialog is a multi-stage project. The Rust",
        "workspace replaces the application shell and compatibility core without",
        "pretending that those subsystems are complete. The Python tree remains in",
        "the repository as the executable specification and fallback implementation.",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    content = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != content:
            print(f"{OUTPUT} is stale", file=__import__("sys").stderr)
            return 1
        print("PARITY.md is current")
    else:
        OUTPUT.write_text(content, encoding="utf-8")
        print(f"wrote {OUTPUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

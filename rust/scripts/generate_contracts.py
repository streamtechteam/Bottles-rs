#!/usr/bin/env python3
"""Generate source-of-truth contracts for the Rust port from the Python app.

This script deliberately reads the existing implementation instead of relying on
a hand-written feature list.  The generated JSON files are embedded into
`bottles-core` and checked by unit tests and CI.
"""
from __future__ import annotations

import argparse
import ast
import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "rust" / "contracts"

# Python modules that have direct or partial Rust counterparts.  This table is
# about *traceability*, not an assertion that all behavior has been ported.
RUST_SURFACES = {
    "bottles/backend/globals.py": ("crates/bottles-core/src/paths.rs", "ported"),
    "bottles/backend/health.py": ("crates/bottles-core/src/health.rs", "partial"),
    "bottles/backend/models/config.py": ("crates/bottles-core/src/config.rs", "ported"),
    "bottles/backend/models/registry_rule.py": ("crates/bottles-core/src/config.rs", "ported"),
    "bottles/backend/managers/data.py": ("crates/bottles-core/src/settings.rs", "ported"),
    "bottles/backend/managers/journal.py": ("crates/bottles-core/src/journal.rs", "ported"),
    "bottles/backend/managers/library.py": ("crates/bottles-core/src/library.rs", "partial"),
    "bottles/backend/managers/importer.py": ("crates/bottles-core/src/manager.rs", "ported"),
    "bottles/backend/managers/manager.py": ("crates/bottles-core/src/manager.rs", "partial"),
    "bottles/backend/managers/registry_rule.py": ("crates/bottles-core/src/registry_rules.rs", "ported"),
    "bottles/backend/state.py": ("crates/bottles-core/src/state.rs", "partial"),
    "bottles/frontend/cli/cli.py": ("crates/bottles-cli/src/main.rs", "partial"),
    "bottles/frontend/main.py": ("crates/bottles-gui/src/main.rs", "partial"),
    "bottles/frontend/windows/window.py": ("crates/bottles-gui/src/ui.rs", "partial"),
}

CLI_STATUS = {
    "info": "ported",
    "list": "ported",
    "programs": "ported",
    "add": "ported",
    "tools": "ported",
    "reg": "ported",
    "reg-rules": "ported",
    "edit": "ported",
    "new": "ported",
    "run": "ported",
    "autostart": "ported",
    "stop": "ported",
    "standalone": "ported",
    "shell": "ported",
    "umu": "partial",
}

GUI_STATUS = {
    "window": "ported",
    "list": "ported",
    "details": "ported",
    "details-bottle": "ported",
    "details-preferences": "ported",
    "details-dependencies": "partial",
    "details-installers": "partial",
    "details-versioning": "partial",
    "details-taskmanager": "partial",
    "details-registry-rules": "ported",
    "library": "ported",
    "preferences": "ported",
    "importer": "ported",
    "loading": "ported",
    "new-bottle-dialog": "ported",
    "dialog-journal": "ported",
    "dialog-rename": "ported",
    "dialog-duplicate": "ported",
    "dialog-launch-options": "ported",
    "dialog-env-vars": "ported",
    "dialog-registry-rules": "ported",
    "help-overlay": "ported",
}


def literal(node: ast.AST | None) -> Any:
    if node is None:
        return None
    try:
        return ast.literal_eval(node)
    except (ValueError, TypeError, SyntaxError):
        return ast.unparse(node)


def dataclass_fields(path: Path, class_name: str) -> list[dict[str, Any]]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    for node in tree.body:
        if not isinstance(node, ast.ClassDef) or node.name != class_name:
            continue
        result: list[dict[str, Any]] = []
        for item in node.body:
            if not isinstance(item, ast.AnnAssign) or not isinstance(item.target, ast.Name):
                continue
            annotation = ast.unparse(item.annotation)
            value = item.value
            factory = False
            if isinstance(value, ast.Call) and getattr(value.func, "id", "") == "field":
                for keyword in value.keywords:
                    if keyword.arg == "default_factory":
                        factory = True
                        expression = ast.unparse(keyword.value)
                        if "list" in expression:
                            default: Any = []
                        elif "dict" in expression:
                            default = {}
                        else:
                            default = "BottleParams()" if "BottleParams" in expression else {}
                        break
                else:
                    default = None
            else:
                default = literal(value)
            result.append(
                {
                    "name": item.target.id,
                    "python_type": annotation,
                    "default": default,
                    "default_factory": factory,
                }
            )
        return result
    raise RuntimeError(f"class {class_name} not found in {path}")


def parse_settings() -> list[dict[str, Any]]:
    path = ROOT / "data" / "com.usebottles.bottles.gschema.xml"
    root = ET.parse(path).getroot()
    keys: list[dict[str, Any]] = []
    for key in root.iter("key"):
        name = key.attrib["name"]
        typ = key.attrib["type"]
        default_node = key.find("default")
        raw = (default_node.text or "").strip() if default_node is not None else ""
        try:
            default = ast.literal_eval(raw)
        except (ValueError, SyntaxError):
            default = raw
        summary = key.findtext("summary") or ""
        description = (key.findtext("description") or "").strip()
        keys.append(
            {
                "name": name,
                "type": typ,
                "default": default,
                "summary": summary,
                "description": description,
            }
        )
    return keys


def call_name(node: ast.AST) -> str | None:
    if isinstance(node, ast.Call):
        return ast.unparse(node.func)
    return None


def parse_cli() -> dict[str, Any]:
    path = ROOT / "bottles" / "frontend" / "cli" / "cli.py"
    tree = ast.parse(path.read_text(encoding="utf-8"))
    parser_vars: dict[str, str] = {}
    commands: list[dict[str, Any]] = []
    globals_: list[dict[str, Any]] = []

    def argument(call: ast.Call) -> dict[str, Any] | None:
        if not call.args:
            return None
        first = literal(call.args[0])
        if not isinstance(first, str):
            return None
        choices: list[Any] = []
        for keyword in call.keywords:
            if keyword.arg == "choices":
                choices = list(literal(keyword.value) or [])
        return {
            "flags": [first] if isinstance(call.args[0], ast.Constant) else [],
            "all_flags": [literal(x) for x in call.args if isinstance(x, ast.Constant)],
            "choices": choices,
            "help": next(
                (literal(k.value) for k in call.keywords if k.arg == "help"), ""
            ),
            "required": next(
                (bool(literal(k.value)) for k in call.keywords if k.arg == "required"),
                False,
            ),
        }

    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
            value = node.value
            if isinstance(target, ast.Name) and isinstance(value, ast.Call):
                name = call_name(value)
                if name and name.endswith("subparsers.add_parser") and value.args:
                    command = literal(value.args[0])
                    if isinstance(command, str):
                        parser_vars[target.id] = command
                        commands.append(
                            {
                                "name": command,
                                "help": next(
                                    (literal(k.value) for k in value.keywords if k.arg == "help"),
                                    "",
                                ),
                                "arguments": [],
                                "status": CLI_STATUS.get(command, "reference-only"),
                            }
                        )
    # A few subparsers are created directly as expression statements (for
    # example `subparsers.add_parser("autostart", ...)`) rather than assigned
    # to a variable.  Capture those too so the contract contains every command.
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not (call_name(node) or "").endswith(
            "subparsers.add_parser"
        ):
            continue
        if not node.args:
            continue
        command = literal(node.args[0])
        if not isinstance(command, str):
            continue
        names = {item["name"] for item in commands}
        if command not in names:
            commands.append(
                {
                    "name": command,
                    "help": next(
                        (literal(k.value) for k in node.keywords if k.arg == "help"),
                        "",
                    ),
                    "arguments": [],
                    "status": CLI_STATUS.get(command, "reference-only"),
                }
            )
    by_name = {item["name"]: item for item in commands}
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and call_name(node) and call_name(node).endswith("add_argument")):
            continue
        parsed = argument(node)
        if parsed is None:
            continue
        owner = node.func.value if isinstance(node.func, ast.Attribute) else None
        if isinstance(owner, ast.Name) and owner.id in parser_vars:
            by_name[parser_vars[owner.id]]["arguments"].append(parsed)
        elif isinstance(owner, ast.Attribute) and owner.attr == "parser":
            globals_.append(parsed)
    return {
        "commands": commands,
        "global_arguments": globals_,
    }


def parse_css_and_ui() -> dict[str, Any]:
    ui_dir = ROOT / "bottles" / "frontend" / "ui"
    screens = []
    for path in sorted(ui_dir.glob("*.blp")):
        stem = path.stem
        screens.append(
            {
                "name": stem,
                "source": path.relative_to(ROOT).as_posix(),
                "kind": "dialog" if stem.startswith("dialog-") else "screen",
                "status": GUI_STATUS.get(stem, "inventory-only"),
                "rust_surface": (
                    "crates/bottles-gui/src/ui.rs"
                    if GUI_STATUS.get(stem, "inventory-only") != "inventory-only"
                    else ""
                ),
            }
        )
    return {"screens": screens}


def collect_modules() -> list[dict[str, Any]]:
    result: list[dict[str, Any]] = []
    for path in sorted((ROOT / "bottles").rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/tests/" in rel or rel.endswith("__init__.py") or rel.endswith("meson.build"):
            continue
        surface, status = RUST_SURFACES.get(rel, ("", "reference-only"))
        result.append(
            {
                "name": path.stem,
                "source": rel,
                "area": rel.split("/")[1] if "/" in rel else "root",
                "status": status,
                "rust_surface": surface,
            }
        )
    return result


def build_contracts() -> dict[str, Any]:
    config_path = ROOT / "bottles" / "backend" / "models" / "config.py"
    return {
        "config_fields.json": {
            "source": config_path.relative_to(ROOT).as_posix(),
            "bottle_config": dataclass_fields(config_path, "BottleConfig"),
            "bottle_params": dataclass_fields(config_path, "BottleParams"),
            "bottle_sandbox_params": dataclass_fields(config_path, "BottleSandboxParams"),
        },
        "settings.json": {
            "source": "data/com.usebottles.bottles.gschema.xml",
            "keys": parse_settings(),
        },
        "cli.json": {
            "source": "bottles/frontend/cli/cli.py",
            **parse_cli(),
        },
        "ui.json": {
            "source": "bottles/frontend/ui",
            **parse_css_and_ui(),
        },
        "modules.json": {
            "source": "bottles",
            "modules": collect_modules(),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="verify generated files are current")
    args = parser.parse_args()
    contracts = build_contracts()
    changed = []
    for filename, payload in contracts.items():
        path = OUT / filename
        content = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                changed.append(filename)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    if args.check:
        if changed:
            print("generated contracts are stale: " + ", ".join(changed), file=sys.stderr)
            return 1
        print("generated contracts are current")
    else:
        print(f"wrote {len(contracts)} contracts to {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

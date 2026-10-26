#!/usr/bin/env python3
"""Generate the GUI's offline catalog snapshot from Bottles' pinned repositories.

The Python application pins three upstream repositories in
`bottles/backend/managers/repository.py`.  This script reads the exact same
commits through GitHub's codeload endpoint and writes a compact JSON snapshot
for the Rust/egui installer, dependency, and component screens.  It requires
PyYAML and network access to codeload.github.com.
"""
from __future__ import annotations

import argparse
import io
import json
import tarfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "rust" / "crates" / "bottles-gui" / "assets" / "catalogs.json"

SOURCES = {
    "installers": {
        "repository": "bottlesdevs/programs",
        "commit": "60a3de90d1a00ddc0e02393e64d1721a789f8af6",
        "index": "index.yml",
    },
    "dependencies": {
        "repository": "bottlesdevs/dependencies",
        "commit": "23bd12fd717c506addb0b05859a5f8635036f225",
        "index": "index.yml",
    },
    "components": {
        "repository": "bottlesdevs/components",
        "commit": "fdfdc568c7ec9744b6feee21b7519b38aeaaae6d",
        "index": "index.yml",
    },
}


def read_index(source: dict[str, str]) -> dict:
    url = (
        f"https://codeload.github.com/{source['repository']}/tar.gz/"
        f"{source['commit']}"
    )
    request = urllib.request.Request(url, headers={"User-Agent": "Bottles-rs"})
    with urllib.request.urlopen(request, timeout=60) as response:
        payload = response.read()
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:gz") as archive:
        member = next(
            member
            for member in archive.getmembers()
            if member.name.endswith("/" + source["index"])
        )
        handle = archive.extractfile(member)
        if handle is None:
            raise RuntimeError(f"Could not read {source['index']}")
        import yaml

        return yaml.safe_load(handle.read().decode("utf-8")) or {}


def as_list(value) -> list[str]:
    if isinstance(value, list):
        return [str(item) for item in value]
    return []


def render() -> str:
    installers = []
    for identifier, metadata in read_index(SOURCES["installers"]).items():
        metadata = metadata or {}
        installers.append(
            {
                "id": str(identifier),
                "name": str(metadata.get("Name", identifier)),
                "description": str(metadata.get("Description", "")),
                "grade": str(metadata.get("Grade", "")),
                "category": str(metadata.get("Category", "")),
                "arch": str(metadata.get("Arch", "win64")),
                "channel": str(metadata.get("Channel", "stable")),
                "runners": as_list(metadata.get("Runners")),
                "icon": str(metadata.get("Icon", "")),
            }
        )
    installers.sort(key=lambda item: item["name"].casefold())

    dependencies = []
    for identifier, metadata in read_index(SOURCES["dependencies"]).items():
        metadata = metadata or {}
        dependencies.append(
            {
                "id": str(identifier),
                "description": str(metadata.get("Description", identifier)),
                "category": str(metadata.get("Category", "")),
            }
        )
    dependencies.sort(key=lambda item: item["description"].casefold())

    components = []
    for identifier, metadata in read_index(SOURCES["components"]).items():
        metadata = metadata or {}
        components.append(
            {
                "id": str(identifier),
                "category": str(metadata.get("Category", "")),
                "subcategory": str(metadata.get("Sub-category", "")),
                "channel": str(metadata.get("Channel", "stable")),
                "date": str(metadata.get("Date", "")),
                "platforms": as_list(metadata.get("Platforms")),
                "architectures": as_list(metadata.get("Architectures")),
            }
        )
    components.sort(key=lambda item: item["id"].casefold())

    document = {
        "sources": {
            name: {
                "repository": source["repository"],
                "commit": source["commit"],
                "url": (
                    f"https://github.com/{source['repository']}/tree/"
                    f"{source['commit']}/{source['index']}"
                ),
            }
            for name, source in SOURCES.items()
        },
        "installers": installers,
        "dependencies": dependencies,
        "components": components,
    }
    return json.dumps(document, indent=2, sort_keys=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    content = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != content:
            print(f"{OUTPUT} is stale", file=__import__("sys").stderr)
            return 1
        print("GUI catalogs are current")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(content, encoding="utf-8")
        print(f"wrote {OUTPUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

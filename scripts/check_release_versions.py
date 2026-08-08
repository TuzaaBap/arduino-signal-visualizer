#!/usr/bin/env python3
"""Fail when a release-visible ASV version differs from the root package version."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(r"^\d+\.\d+\.\d+$")


def read_json(relative_path: str) -> object:
    return json.loads((ROOT / relative_path).read_text(encoding="utf-8"))


def read_toml(relative_path: str) -> dict[str, object]:
    with (ROOT / relative_path).open("rb") as source:
        return tomllib.load(source)


def read_library_version() -> str:
    properties = ROOT / "firmware/ArduinoSignalVisualizer/library.properties"
    for line in properties.read_text(encoding="utf-8").splitlines():
        if line.startswith("version="):
            return line.split("=", 1)[1].strip()
    raise ValueError(f"version is missing from {properties}")


def require_regex(relative_path: str, pattern: str, description: str) -> str:
    content = (ROOT / relative_path).read_text(encoding="utf-8")
    match = re.search(pattern, content)
    if match is None:
        raise ValueError(f"could not find {description} in {relative_path}")
    return ".".join(match.groups()) if len(match.groups()) > 1 else match.group(1)


def collect_versions() -> dict[str, str]:
    root_package = read_json("package.json")
    desktop_package = read_json("desktop/package.json")
    tauri_config = read_json("desktop/src-tauri/tauri.conf.json")
    package_lock = read_json("package-lock.json")
    cargo_workspace = read_toml("Cargo.toml")
    cargo_lock = read_toml("Cargo.lock")

    lock_packages = {
        package["name"]: package["version"]
        for package in cargo_lock["package"]
        if package["name"] in {"asv-desktop", "asv-protocol"}
    }

    return {
        "root package.json": root_package["version"],
        "desktop/package.json": desktop_package["version"],
        "desktop/src-tauri/tauri.conf.json": tauri_config["version"],
        "Cargo.toml workspace": cargo_workspace["workspace"]["package"]["version"],
        "Cargo.lock asv-desktop": lock_packages["asv-desktop"],
        "Cargo.lock asv-protocol": lock_packages["asv-protocol"],
        "package-lock.json root": package_lock["packages"][""]["version"],
        "package-lock.json desktop": package_lock["packages"]["desktop"]["version"],
        "Arduino library": read_library_version(),
        "desktop UI": require_regex(
            "desktop/src/App.tsx",
            r'<Metric label="App" value="(\d+\.\d+\.\d+)"\s*/>',
            "application version metric",
        ),
        "firmware hello": require_regex(
            "firmware/ArduinoSignalVisualizer/src/ArduinoSignalVisualizer.cpp",
            r"(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*// firmware",
            "firmware hello version",
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--print-version",
        action="store_true",
        help="print only the validated release version",
    )
    args = parser.parse_args()

    try:
        versions = collect_versions()
    except (KeyError, TypeError, ValueError) as error:
        print(f"Version check failed: {error}", file=sys.stderr)
        return 1

    expected = versions["root package.json"]
    if not SEMVER.fullmatch(expected):
        print(f"Version check failed: {expected!r} is not X.Y.Z", file=sys.stderr)
        return 1

    mismatches = {
        location: version
        for location, version in versions.items()
        if version != expected
    }
    if mismatches:
        print(f"Version check failed: expected {expected}", file=sys.stderr)
        for location, version in mismatches.items():
            print(f"  {location}: {version}", file=sys.stderr)
        return 1

    if args.print_version:
        print(expected)
    else:
        print(f"Release version {expected} is consistent across {len(versions)} sources.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

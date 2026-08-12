#!/usr/bin/env python3
"""Audit a complete ASV stable-release asset directory and write checksums."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlparse


REQUIRED_UPDATER_PLATFORMS = {
    "windows-x86_64",
    "darwin-aarch64",
    "darwin-x86_64",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def audit(asset_dir: Path, version: str) -> Path:
    files = sorted(path for path in asset_dir.iterdir() if path.is_file())
    names = {path.name for path in files}
    require(files, "release asset directory is empty")
    require(any(name.lower().endswith("-setup.exe") for name in names), "NSIS setup executable is missing")
    require(any(name.lower().endswith(".msi") for name in names), "Windows MSI is missing")
    require(sum(name.lower().endswith(".dmg") for name in names) >= 2, "both macOS DMGs are required")

    library_name = f"ArduinoSignalVisualizer-{version}.zip"
    require(library_name in names, f"Arduino library is missing: {library_name}")
    require(f"{library_name}.sha256" in names, "Arduino library checksum is missing")

    manifest_path = asset_dir / "latest.json"
    require(manifest_path.is_file(), "latest.json is missing")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(manifest.get("version") == version, "updater manifest version does not match the release")
    platforms = manifest.get("platforms")
    require(isinstance(platforms, dict), "updater manifest platforms are missing")
    missing_platforms = REQUIRED_UPDATER_PLATFORMS.difference(platforms)
    require(not missing_platforms, f"updater manifest is missing: {', '.join(sorted(missing_platforms))}")

    for platform in sorted(REQUIRED_UPDATER_PLATFORMS):
        entry = platforms[platform]
        require(isinstance(entry, dict), f"invalid updater entry for {platform}")
        signature = entry.get("signature")
        url = entry.get("url")
        require(isinstance(signature, str) and len(signature.strip()) >= 64, f"signature is missing for {platform}")
        require(isinstance(url, str) and url.startswith("https://"), f"download URL is invalid for {platform}")
        parsed_url = urlparse(url)
        referenced_name = unquote(Path(parsed_url.path).name)
        is_github_asset_api = (
            parsed_url.hostname == "api.github.com"
            and re.search(r"/releases/assets/\d+$", parsed_url.path) is not None
        )
        require(
            referenced_name in names or is_github_asset_api,
            f"updater asset URL is not a downloaded file or GitHub release asset for {platform}: {url}",
        )

    checksum_path = asset_dir / "SHA256SUMS.txt"
    checksum_lines = [
        f"{sha256(path)}  {path.name}"
        for path in files
        if path.name != checksum_path.name
    ]
    checksum_path.write_text("\n".join(checksum_lines) + "\n", encoding="ascii", newline="\n")
    return checksum_path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-dir", type=Path, required=True)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    try:
        checksum_path = audit(args.asset_dir.resolve(), args.version)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"Release asset audit failed: {error}", file=sys.stderr)
        return 1
    print(f"Release asset audit passed; wrote {checksum_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

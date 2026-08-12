#!/usr/bin/env python3
"""Regression tests for the stable-release asset audit."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest

from audit_release_assets import audit


class ReleaseAssetAuditTests(unittest.TestCase):
    def create_assets(self, root: Path) -> None:
        asset_names = (
            "Arduino-Signal-Visualizer-0.6.0-windows-x86_64-setup.exe",
            "Arduino-Signal-Visualizer-0.6.0-windows-x86_64.msi",
            "Arduino-Signal-Visualizer-0.6.0-darwin-aarch64.dmg",
            "Arduino-Signal-Visualizer-0.6.0-darwin-x86_64.dmg",
            "Arduino-Signal-Visualizer-0.6.0-windows-x86_64.nsis.zip",
            "Arduino-Signal-Visualizer-0.6.0-darwin-aarch64.app.tar.gz",
            "Arduino-Signal-Visualizer-0.6.0-darwin-x86_64.app.tar.gz",
            "ArduinoSignalVisualizer-0.6.0.zip",
            "ArduinoSignalVisualizer-0.6.0.zip.sha256",
        )
        for name in asset_names:
            (root / name).write_bytes(f"test asset {name}".encode())

        platforms = {
            "windows-x86_64": "Arduino-Signal-Visualizer-0.6.0-windows-x86_64.nsis.zip",
            "darwin-aarch64": "Arduino-Signal-Visualizer-0.6.0-darwin-aarch64.app.tar.gz",
            "darwin-x86_64": "Arduino-Signal-Visualizer-0.6.0-darwin-x86_64.app.tar.gz",
        }
        manifest = {
            "version": "0.6.0",
            "platforms": {
                platform: {
                    "signature": "s" * 64,
                    "url": f"https://github.test/releases/{asset_name}",
                }
                for platform, asset_name in platforms.items()
            },
        }
        (root / "latest.json").write_text(json.dumps(manifest), encoding="utf-8")

    def test_complete_release_writes_checksums(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.create_assets(root)

            checksum_path = audit(root, "0.6.0")

            contents = checksum_path.read_text(encoding="ascii")
            self.assertIn("latest.json", contents)
            self.assertIn("ArduinoSignalVisualizer-0.6.0.zip", contents)

    def test_missing_updater_platform_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.create_assets(root)
            manifest_path = root / "latest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            del manifest["platforms"]["darwin-x86_64"]
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "darwin-x86_64"):
                audit(root, "0.6.0")

    def test_github_release_asset_api_urls_are_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.create_assets(root)
            manifest_path = root / "latest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            for index, entry in enumerate(manifest["platforms"].values(), start=100):
                entry["url"] = (
                    "https://api.github.com/repos/TuzaaBap/"
                    f"arduino-signal-visualizer/releases/assets/{index}"
                )
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            checksum_path = audit(root, "0.6.0")

            self.assertTrue(checksum_path.is_file())


if __name__ == "__main__":
    unittest.main()

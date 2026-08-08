#!/usr/bin/env python3
"""Create a deterministic Arduino IDE-installable library ZIP."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import sys
import zipfile


LIBRARY_NAME = "ArduinoSignalVisualizer"
REQUIRED_FILES = (
    "library.properties",
    "LICENSE",
    "README.md",
    "keywords.txt",
    "src/ASVInstrumented.h",
    "src/ArduinoSignalVisualizer.h",
    "src/ArduinoSignalVisualizer.cpp",
    "src/AsvProtocol.h",
    "src/AsvProtocol.cpp",
    "examples/GpioDemo/GpioDemo.ino",
    "examples/AdcDemo/AdcDemo.ino",
    "examples/PwmDemo/PwmDemo.ino",
    "examples/TransparentSerialDemo/TransparentSerialDemo.ino",
)


def read_properties(path: Path) -> dict[str, str]:
    properties: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if not separator:
            raise ValueError(f"Malformed property line: {raw_line}")
        properties[key.strip()] = value.strip()
    return properties


def package(output_dir: Path) -> tuple[Path, Path, str]:
    repository_root = Path(__file__).resolve().parent.parent
    library_root = repository_root / "firmware" / LIBRARY_NAME

    missing = [name for name in REQUIRED_FILES if not (library_root / name).is_file()]
    if missing:
        raise FileNotFoundError(f"Library is incomplete: {', '.join(missing)}")

    properties = read_properties(library_root / "library.properties")
    if properties.get("name") != LIBRARY_NAME:
        raise ValueError("library.properties name does not match the ZIP root folder")
    version = properties.get("version")
    if not version:
        raise ValueError("library.properties is missing a version")

    output_dir.mkdir(parents=True, exist_ok=True)
    archive_path = output_dir / f"{LIBRARY_NAME}-{version}.zip"

    source_files = sorted(
        path
        for path in library_root.rglob("*")
        if path.is_file()
        and path.name not in {".DS_Store", ".development"}
        and path.suffix not in {".pyc"}
        and "__pycache__" not in path.parts
    )
    with zipfile.ZipFile(
        archive_path,
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as archive:
        for source_path in source_files:
            relative_path = source_path.relative_to(library_root)
            archive_name = (Path(LIBRARY_NAME) / relative_path).as_posix()
            entry = zipfile.ZipInfo(archive_name, date_time=(2026, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, source_path.read_bytes(), compresslevel=9)

    digest = hashlib.sha256(archive_path.read_bytes()).hexdigest().upper()
    checksum_path = archive_path.with_suffix(".zip.sha256")
    checksum_path.write_text(
        f"{digest}  {archive_path.name}\n",
        encoding="ascii",
        newline="\n",
    )
    return archive_path, checksum_path, digest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("outputs/release"),
        help="Directory for the ZIP and SHA-256 file",
    )
    args = parser.parse_args()
    try:
        archive_path, checksum_path, digest = package(args.output_dir.resolve())
    except (OSError, ValueError) as error:
        print(f"Packaging failed: {error}", file=sys.stderr)
        return 1

    print(f"Created {archive_path}")
    print(f"Created {checksum_path}")
    print(f"SHA-256 {digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

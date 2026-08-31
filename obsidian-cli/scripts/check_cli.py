#!/usr/bin/env python3
"""Locate the registered Obsidian CLI without launching a known GUI wrapper."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def candidates():
    paths = [Path.home() / ".local" / "bin" / "obsidian"]
    discovered = shutil.which("obsidian")
    if discovered:
        paths.append(Path(discovered))
    seen = set()
    for path in paths:
        try:
            key = path.resolve()
        except OSError:
            key = path
        if key not in seen:
            seen.add(key)
            yield path


def is_gui_wrapper(path: Path) -> bool:
    try:
        if path.stat().st_size > 65536:
            return False
        sample = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return False
    lowered = sample.lower()
    return "electron" in lowered and "app.asar" in lowered


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()
    for path in candidates():
        if not path.is_file() or not os.access(path, os.X_OK):
            continue
        if is_gui_wrapper(path):
            print(f"SKIP GUI wrapper: {path}")
            continue
        try:
            result = subprocess.run(
                [str(path), "version"],
                text=True,
                capture_output=True,
                timeout=5,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            print(f"SKIP unusable candidate {path}: {exc}")
            continue
        output = (result.stdout or result.stderr).strip()
        if result.returncode == 0 and output:
            print(f"CLI {path}: {output}")
            return 0
        print(f"SKIP non-CLI candidate {path}: exit {result.returncode}")
    print(
        "Obsidian CLI unavailable. Enable Settings → General → Command line interface; "
        "on Linux ensure ~/.local/bin precedes the GUI launcher in PATH."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Locate the registered Obsidian CLI without launching a known GUI wrapper."""

from __future__ import annotations

import argparse
import json
import os
import re
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


def recognizable_version(output: str) -> bool:
    return bool(re.fullmatch(
        r"(?:Obsidian\s+)?\d+\.\d+(?:\.\d+)?(?:[-+][\w.-]+)?"
        r"(?:\s+\(installer\s+[^\n()]+\))?", output.strip(), re.I
    ))


def recognizable_help(output: str) -> bool:
    # Require Obsidian identity plus several command entries, not arbitrary logs.
    commands = re.findall(r"^\s*(read|search|vault)\b", output, re.M)
    return "obsidian" in output.lower() and len(set(commands)) == 3


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit the verified executable path as JSON")
    args = parser.parse_args()
    def diagnostic(message):
        print(message, file=sys.stderr if args.json else sys.stdout)
    for path in candidates():
        if not path.is_file() or not os.access(path, os.X_OK):
            continue
        if is_gui_wrapper(path):
            diagnostic(f"SKIP GUI wrapper: {path}")
            continue
        try:
            result = subprocess.run(
                [str(path), "version"],
                text=True,
                capture_output=True,
                timeout=5,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            diagnostic(f"SKIP unusable candidate {path}: {exc}")
            continue
        output = (result.stdout or result.stderr).strip()
        if result.returncode != 0 or not recognizable_version(output):
            diagnostic(f"SKIP non-CLI candidate {path}: unrecognized version response")
            continue
        try:
            help_result = subprocess.run(
                [str(path), "help"], text=True, capture_output=True, timeout=5
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            diagnostic(f"SKIP unusable help response {path}: {exc}")
            continue
        if help_result.returncode != 0 or not recognizable_help(help_result.stdout):
            diagnostic(f"SKIP non-CLI candidate {path}: unrecognized help response")
            continue
        # Preserve the tested invocation path, including a registered symlink.
        executable = str(path.absolute())
        if args.json:
            print(json.dumps({"executable": executable, "version": output}))
        else:
            print(f"CLI {executable}: {output}")
        return 0
    diagnostic(
        "Obsidian CLI unavailable. Enable Settings → General → Command line interface; "
        "on Linux ensure ~/.local/bin precedes the GUI launcher in PATH."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())

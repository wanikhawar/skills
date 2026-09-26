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


def cli_enabled() -> bool:
    """Return True when Obsidian's own config records the CLI as enabled."""
    config_home = Path(os.environ.get("XDG_CONFIG_HOME") or Path.home() / ".config")
    try:
        data = json.loads((config_home / "obsidian" / "obsidian.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return False
    return isinstance(data, dict) and data.get("cli") is True


def obsidian_running() -> bool:
    """Return True when an Obsidian app process is already running (Linux /proc)."""
    proc = Path("/proc")
    if not proc.is_dir():
        return False
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            cmdline = (entry / "cmdline").read_bytes().lower()
        except OSError:
            continue
        if b"obsidian" in cmdline and b"app.asar" in cmdline:
            return True
    return False


def wrapper_probe_allowed() -> bool:
    # A packaged launcher (e.g. Arch/AUR) forwards CLI commands to the running app
    # once the CLI is enabled. Probe it only then, so the check cannot open a window.
    return cli_enabled() and obsidian_running()


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
            if not wrapper_probe_allowed():
                diagnostic(f"SKIP GUI wrapper: {path} (CLI not enabled or Obsidian not running)")
                continue
            diagnostic(f"PROBE GUI wrapper: {path} (CLI enabled and Obsidian running)")
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
        "Obsidian CLI unavailable. Enable Settings → General → Command line interface "
        "and keep Obsidian running; on Linux ensure ~/.local/bin precedes the GUI launcher "
        "in PATH when a separate CLI is registered there."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())

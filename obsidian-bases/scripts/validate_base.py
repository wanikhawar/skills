#!/usr/bin/env python3
"""Static validator for Obsidian .base files.

This checks structure and common stale formula patterns. Obsidian remains the
authority for formula execution and plugin-specific view configuration.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError as exc:  # pragma: no cover - dependency failure path
    raise SystemExit("PyYAML is required: install the 'pyyaml' package") from exc


BUILTIN_VIEWS = {"table", "cards", "list", "map"}
STALE_DURATION_FIELD = re.compile(
    r"\.(?:days|hours|minutes|seconds|milliseconds)(?:\b|\()"
)
FORMULA_REF = re.compile(r"\bformula\.([A-Za-z_][A-Za-z0-9_-]*)\b")


def iter_strings(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, list):
        for item in value:
            yield from iter_strings(item)
    elif isinstance(value, dict):
        for key, item in value.items():
            if isinstance(key, str):
                yield key
            yield from iter_strings(item)


def validate_filter(value, location, errors):
    if isinstance(value, str):
        return
    if not isinstance(value, dict):
        errors.append(f"{location}: filter must be a string or mapping")
        return
    if len(value) != 1:
        errors.append(f"{location}: filter mapping must contain exactly one of and/or/not")
    for key, children in value.items():
        if key not in {"and", "or", "not"}:
            errors.append(f"{location}: unknown filter operator {key!r}")
            continue
        if not isinstance(children, list):
            errors.append(f"{location}.{key}: value must be a list")
            continue
        for index, child in enumerate(children):
            validate_filter(child, f"{location}.{key}[{index}]", errors)


def validate(path: Path):
    errors = []
    warnings = []
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, yaml.YAMLError) as exc:
        return [f"cannot parse YAML: {exc}"], warnings

    if not isinstance(data, dict):
        return ["top level must be a YAML mapping"], warnings

    formulas = data.get("formulas", {})
    if formulas is None:
        formulas = {}
    if not isinstance(formulas, dict):
        errors.append("formulas must be a mapping")
        formulas = {}
    else:
        for name, expression in formulas.items():
            if not isinstance(name, str) or not name:
                errors.append("formula names must be non-empty strings")
            if not isinstance(expression, str):
                errors.append(f"formula {name!r} must be a string expression")

    if "filters" in data:
        validate_filter(data["filters"], "filters", errors)

    views = data.get("views")
    if not isinstance(views, list) or not views:
        errors.append("views must be a non-empty list")
        views = []

    seen_names = set()
    for index, view in enumerate(views):
        where = f"views[{index}]"
        if not isinstance(view, dict):
            errors.append(f"{where}: view must be a mapping")
            continue
        view_type = view.get("type")
        name = view.get("name")
        if not isinstance(view_type, str) or not view_type:
            errors.append(f"{where}: missing string type")
        elif view_type not in BUILTIN_VIEWS:
            warnings.append(f"{where}: {view_type!r} is plugin-defined; verify the plugin")
        if not isinstance(name, str) or not name:
            errors.append(f"{where}: missing non-empty string name")
        elif name in seen_names:
            errors.append(f"{where}: duplicate view name {name!r}")
        else:
            seen_names.add(name)
        if "filters" in view:
            validate_filter(view["filters"], f"{where}.filters", errors)
        order = view.get("order", [])
        if not isinstance(order, list) or not all(isinstance(v, str) for v in order):
            errors.append(f"{where}.order: must be a list of property names")
        group_by = view.get("groupBy")
        if group_by is not None:
            if not isinstance(group_by, dict):
                errors.append(f"{where}.groupBy: must be a mapping")
            else:
                if not isinstance(group_by.get("property"), str):
                    errors.append(f"{where}.groupBy.property: must be a property name")
                if group_by.get("direction") not in {"ASC", "DESC"}:
                    errors.append(f"{where}.groupBy.direction: must be ASC or DESC")

    defined = set(formulas)
    for expression in iter_strings(data):
        for referenced in FORMULA_REF.findall(expression):
            if referenced not in defined:
                errors.append(f"undefined formula reference: formula.{referenced}")
        if STALE_DURATION_FIELD.search(expression):
            errors.append(
                "stale duration-field syntax found; date subtraction returns milliseconds"
            )

    if path.suffix != ".base":
        warnings.append("file does not use the .base extension")

    return sorted(set(errors)), sorted(set(warnings))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="+", type=Path)
    args = parser.parse_args()
    failed = False
    for path in args.files:
        errors, warnings = validate(path)
        for message in warnings:
            print(f"WARNING {path}: {message}")
        for message in errors:
            print(f"ERROR {path}: {message}")
        if errors:
            failed = True
        else:
            print(f"OK {path}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Validate JSON Canvas structure and Khawar-vault layout conventions."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path, PurePosixPath


NODE_TYPES = {"text", "file", "link", "group"}
SIDES = {"top", "right", "bottom", "left"}
ENDS = {"none", "arrow"}
HEX_ID = re.compile(r"^[0-9a-f]{16}$")
HEX_COLOR = re.compile(r"^#[0-9A-Fa-f]{6}$")


def valid_color(value):
    return value in {"1", "2", "3", "4", "5", "6"} or (
        isinstance(value, str) and bool(HEX_COLOR.fullmatch(value))
    )


def rect(node):
    return (node["x"], node["y"], node["x"] + node["width"], node["y"] + node["height"])


def overlaps(a, b):
    return not (a[2] <= b[0] or b[2] <= a[0] or a[3] <= b[1] or b[3] <= a[1])


def contains(outer, inner):
    return outer[0] <= inner[0] and outer[1] <= inner[1] and outer[2] >= inner[2] and outer[3] >= inner[3]


def validate(path: Path, strict_layout=False):
    errors = []
    warnings = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        return [f"cannot parse JSON: {exc}"], warnings
    if not isinstance(data, dict):
        return ["top level must be an object"], warnings
    nodes = data.get("nodes", [])
    edges = data.get("edges", [])
    if not isinstance(nodes, list) or not isinstance(edges, list):
        return ["nodes and edges must be arrays"], warnings

    all_ids = set()
    node_ids = set()
    valid_nodes = []
    for index, node in enumerate(nodes):
        where = f"nodes[{index}]"
        if not isinstance(node, dict):
            errors.append(f"{where}: must be an object")
            continue
        node_id = node.get("id")
        if not isinstance(node_id, str) or not node_id:
            errors.append(f"{where}: missing non-empty string id")
        elif node_id in all_ids:
            errors.append(f"{where}: duplicate id {node_id!r}")
        else:
            all_ids.add(node_id)
            node_ids.add(node_id)
            if not HEX_ID.fullmatch(node_id):
                warnings.append(f"{where}: id is valid but not the local 16-hex convention")
        node_type = node.get("type")
        if node_type not in NODE_TYPES:
            warnings.append(f"{where}: unknown node type {node_type!r}; verify its extension")
        for key in ("x", "y", "width", "height"):
            if not isinstance(node.get(key), int):
                errors.append(f"{where}: {key} must be an integer")
        if isinstance(node.get("width"), int) and node["width"] <= 0:
            errors.append(f"{where}: width must be positive")
        if isinstance(node.get("height"), int) and node["height"] <= 0:
            errors.append(f"{where}: height must be positive")
        if "color" in node and not valid_color(node["color"]):
            errors.append(f"{where}: color must be a preset 1-6 or #RRGGBB")
        required = {"text": "text", "file": "file", "link": "url"}.get(node_type)
        if required and not isinstance(node.get(required), str):
            errors.append(f"{where}: {node_type} node requires string {required}")
        if node_type == "file" and isinstance(node.get("file"), str):
            file_path = node["file"]
            if (
                Path(file_path).is_absolute()
                or PurePosixPath(file_path).is_absolute()
                or re.match(r"^[A-Za-z]:[\\/]", file_path)
            ):
                errors.append(f"{where}: file path must be vault-relative")
            if "\\" in file_path:
                errors.append(f"{where}: file path must use forward slashes")
            if file_path.lower().endswith(".md"):
                warnings.append(f"{where}: Markdown note cards should be raw-wikilink text nodes")
        if "subpath" in node and (
            not isinstance(node["subpath"], str) or not node["subpath"].startswith("#")
        ):
            errors.append(f"{where}: subpath must be a string beginning with #")
        if node_type == "group" and "backgroundStyle" in node and node["backgroundStyle"] not in {
            "cover",
            "ratio",
            "repeat",
        }:
            errors.append(f"{where}: invalid backgroundStyle {node['backgroundStyle']!r}")
        if all(isinstance(node.get(k), int) for k in ("x", "y", "width", "height")):
            valid_nodes.append((index, node))

    for index, edge in enumerate(edges):
        where = f"edges[{index}]"
        if not isinstance(edge, dict):
            errors.append(f"{where}: must be an object")
            continue
        edge_id = edge.get("id")
        if not isinstance(edge_id, str) or not edge_id:
            errors.append(f"{where}: missing non-empty string id")
        elif edge_id in all_ids:
            errors.append(f"{where}: duplicate id {edge_id!r}")
        else:
            all_ids.add(edge_id)
            if not HEX_ID.fullmatch(edge_id):
                warnings.append(f"{where}: id is valid but not the local 16-hex convention")
        for key in ("fromNode", "toNode"):
            if edge.get(key) not in node_ids:
                errors.append(f"{where}: {key} does not reference an existing node")
        for key in ("fromSide", "toSide"):
            if key in edge and edge[key] not in SIDES:
                errors.append(f"{where}: invalid {key} {edge[key]!r}")
        for key in ("fromEnd", "toEnd"):
            if key in edge and edge[key] not in ENDS:
                errors.append(f"{where}: invalid {key} {edge[key]!r}")
        if "color" in edge and not valid_color(edge["color"]):
            errors.append(f"{where}: color must be a preset 1-6 or #RRGGBB")

    for group_index, group in valid_nodes:
        if group.get("type") != "group":
            continue
        group_rect = rect(group)
        for child_index, child in valid_nodes:
            if child_index == group_index or child.get("type") == "group":
                continue
            if contains(group_rect, rect(child)) and group_index > child_index:
                warnings.append(
                    f"nodes[{group_index}]: group contains nodes[{child_index}] but appears above it in z-order"
                )

    if strict_layout:
        ordinary = [(i, n) for i, n in valid_nodes if n.get("type") != "group"]
        for pos, (left_index, left) in enumerate(ordinary):
            for right_index, right in ordinary[pos + 1 :]:
                if overlaps(rect(left), rect(right)):
                    errors.append(f"nodes[{left_index}] overlaps nodes[{right_index}]")

    return sorted(set(errors)), sorted(set(warnings))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict-layout", action="store_true")
    parser.add_argument("files", nargs="+", type=Path)
    args = parser.parse_args()
    failed = False
    for path in args.files:
        errors, warnings = validate(path, args.strict_layout)
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

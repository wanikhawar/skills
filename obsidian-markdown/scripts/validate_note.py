#!/usr/bin/env python3
"""Conservative static checks for Obsidian Markdown notes."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError as exc:  # pragma: no cover
    raise SystemExit("PyYAML is required: install the 'pyyaml' package") from exc


FENCE = re.compile(r"^\s*(`{3,}|~{3,})(.*)$")
TABLE_SEPARATOR = re.compile(
    r"^\s*\|?\s*:?-{3,}:?\s*(?:\|\s*:?-{3,}:?\s*)+\|?\s*$"
)


def strip_fenced_blocks(lines):
    visible = []
    opener = None
    for line_number, line in enumerate(lines, 1):
        match = FENCE.match(line)
        if opener is None:
            if match:
                opener = (match.group(1)[0], len(match.group(1)), line_number)
                visible.append("")
            else:
                visible.append(line)
        else:
            char, length, _ = opener
            stripped = line.lstrip()
            if re.fullmatch(re.escape(char) + "{" + str(length) + r",}\s*", stripped):
                opener = None
            visible.append("")
    return visible, opener


def strip_inline_code(text):
    """Mask matched equal-length backtick runs, preserving line positions."""
    runs = list(re.finditer(r"`+", text))
    result = list(text)
    index = 0
    while index < len(runs):
        opening = runs[index]
        # An escaped backtick outside a code span does not open one.
        prefix = text[:opening.start()]
        if (len(prefix) - len(prefix.rstrip("\\"))) % 2:
            index += 1
            continue
        closing = next((j for j in range(index + 1, len(runs))
                        if len(runs[j].group()) == len(opening.group())), None)
        if closing is None:
            index += 1
            continue
        for position in range(opening.start(), runs[closing].end()):
            if result[position] != "\n":
                result[position] = " "
        index = closing + 1
    return "".join(result)


def unescaped_pipe_count(line):
    count = 0
    escaped = False
    for char in line:
        if escaped:
            escaped = False
        elif char == "\\":
            escaped = True
        elif char == "|":
            count += 1
    return count


def validate(path: Path):
    errors = []
    warnings = []
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        return [f"cannot read file: {exc}"], warnings

    lines = text.splitlines()
    body_start = 0
    if lines and lines[0].strip() == "---":
        end = next((i for i in range(1, len(lines)) if lines[i].strip() == "---"), None)
        if end is None:
            errors.append("frontmatter opens with --- but has no closing ---")
        else:
            body_start = end + 1
            try:
                data = yaml.safe_load("\n".join(lines[1:end]))
                if data is not None and not isinstance(data, dict):
                    errors.append("frontmatter must be a YAML mapping")
            except yaml.YAMLError as exc:
                errors.append(f"invalid frontmatter YAML: {exc}")

    visible, open_fence = strip_fenced_blocks(lines[body_start:])
    if open_fence:
        errors.append(f"unclosed code fence beginning near body line {open_fence[2]}")
    visible_text = strip_inline_code("\n".join(visible))

    if "\\[" in visible_text or "\\]" in visible_text:
        errors.append("bracket-delimited display math found; use $$ delimiters")
    if visible_text.count("$$") % 2:
        errors.append("unbalanced $$ display-math delimiters")
    math_removed = re.sub(r"\$\$.*?\$\$", "", visible_text, flags=re.S)
    unescaped_dollars = len(re.findall(r"(?<!\\)\$(?!\$)", math_removed))
    if unescaped_dollars % 2:
        warnings.append("odd number of inline $ delimiters; inspect for math/currency ambiguity")

    if visible_text.count("[[") != visible_text.count("]]" ):
        errors.append("unbalanced wikilink delimiters")

    body_lines = visible
    for index, line in enumerate(body_lines):
        if not TABLE_SEPARATOR.match(line):
            continue
        expected = unescaped_pipe_count(line)
        candidates = []
        if index > 0:
            candidates.append((index, body_lines[index - 1]))
        cursor = index + 1
        while cursor < len(body_lines) and "|" in body_lines[cursor] and body_lines[cursor].strip():
            candidates.append((cursor + 1, body_lines[cursor]))
            cursor += 1
        for line_no, candidate in candidates:
            found = unescaped_pipe_count(candidate)
            if found != expected:
                errors.append(
                    f"table near body line {index + 1}: line {line_no} has {found} "
                    f"unescaped pipes; expected {expected}"
                )

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

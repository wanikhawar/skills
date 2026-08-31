#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pymupdf>=1.27,<2",
#   "pypdf>=6.6,<7",
# ]
# ///
"""Deterministic PDF inspection, extraction, rendering, and page operations."""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

import pymupdf
from pypdf import PdfReader, PdfWriter


def parse_pages(spec: str | None, count: int) -> list[int]:
    if not spec:
        return list(range(count))
    selected: list[int] = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            start_text, end_text = part.split("-", 1)
            start, end = int(start_text), int(end_text)
            if start > end:
                raise ValueError(f"descending page range is not allowed: {part}")
            pages = range(start, end + 1)
        else:
            pages = [int(part)]
        for page in pages:
            if page < 1 or page > count:
                raise ValueError(f"page {page} outside valid range 1-{count}")
            zero_based = page - 1
            if zero_based not in selected:
                selected.append(zero_based)
    if not selected:
        raise ValueError("page selection is empty")
    return selected


def require_input(path: Path) -> None:
    if not path.is_file():
        raise ValueError(f"input PDF does not exist: {path}")


def prepare_output(path: Path, overwrite: bool) -> None:
    if path.exists() and not overwrite:
        raise ValueError(f"output exists; pass --overwrite only with authorization: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)


def atomic_pdf_write(writer: PdfWriter, output: Path, overwrite: bool) -> None:
    prepare_output(output, overwrite)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{output.name}.", suffix=".tmp", dir=output.parent
    )
    try:
        with os.fdopen(descriptor, "wb") as stream:
            writer.write(stream)
        os.replace(temporary_name, output)
    except Exception:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def inspect_pdf(path: Path) -> dict:
    require_input(path)
    reader = PdfReader(path)
    document = pymupdf.open(path)
    result = {
        "path": str(path.resolve()),
        "size_bytes": path.stat().st_size,
        "encrypted": bool(reader.is_encrypted or document.needs_pass),
        "pages": document.page_count,
        "metadata": {},
        "page_details": [],
    }
    if not reader.is_encrypted and reader.metadata:
        result["metadata"] = {
            str(key).lstrip("/"): str(value) for key, value in reader.metadata.items() if value is not None
        }
    if document.needs_pass:
        result["text_access"] = "password required"
        document.close()
        return result
    text_pages = 0
    for index, page in enumerate(document):
        text_chars = len(page.get_text("text").strip())
        if text_chars:
            text_pages += 1
        result["page_details"].append(
            {
                "page": index + 1,
                "width_points": round(page.rect.width, 2),
                "height_points": round(page.rect.height, 2),
                "rotation": page.rotation,
                "text_characters": text_chars,
                "images": len(page.get_images(full=True)),
            }
        )
    result["pages_with_text"] = text_pages
    result["likely_scanned"] = bool(result["pages"] and text_pages == 0)
    document.close()
    return result


def command_inspect(args) -> None:
    print(json.dumps(inspect_pdf(args.input), indent=2, ensure_ascii=False))


def command_extract(args) -> None:
    require_input(args.input)
    document = pymupdf.open(args.input)
    if document.needs_pass:
        raise ValueError("PDF requires a password")
    pages = parse_pages(args.pages, document.page_count)
    chunks = []
    for index in pages:
        text = document[index].get_text("text", sort=True).rstrip()
        chunks.append(f"## PDF page {index + 1}\n\n{text}\n")
    output = "\n".join(chunks)
    if args.output:
        prepare_output(args.output, args.overwrite)
        args.output.write_text(output, encoding="utf-8")
    else:
        print(output, end="")


def command_render(args) -> None:
    require_input(args.input)
    document = pymupdf.open(args.input)
    if document.needs_pass:
        raise ValueError("PDF requires a password")
    pages = parse_pages(args.pages, document.page_count)
    if args.dpi <= 0:
        raise ValueError("DPI must be positive")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    scale = args.dpi / 72
    matrix = pymupdf.Matrix(scale, scale)
    outputs = [args.output_dir / f"page-{index + 1:04d}.png" for index in pages]
    if not args.overwrite:
        existing = [path for path in outputs if path.exists()]
        if existing:
            raise ValueError(f"render output exists: {existing[0]}")
    for index, output in zip(pages, outputs):
        pixmap = document[index].get_pixmap(matrix=matrix, alpha=False)
        pixmap.save(output)
        print(output)


def copy_metadata(reader: PdfReader, writer: PdfWriter) -> None:
    if reader.metadata:
        clean = {str(k): str(v) for k, v in reader.metadata.items() if v is not None}
        if clean:
            writer.add_metadata(clean)


def command_merge(args) -> None:
    output_resolved = args.output.resolve()
    if any(source.resolve() == output_resolved for source in args.inputs):
        raise ValueError("merge output must not be one of its inputs")
    writer = PdfWriter()
    for source in args.inputs:
        require_input(source)
        reader = PdfReader(source)
        if reader.is_encrypted:
            raise ValueError(f"encrypted input is not supported without explicit decryption: {source}")
        for page in reader.pages:
            writer.add_page(page)
    atomic_pdf_write(writer, args.output, args.overwrite)


def command_extract_pages(args) -> None:
    require_input(args.input)
    reader = PdfReader(args.input)
    if reader.is_encrypted:
        raise ValueError("encrypted input is not supported without explicit decryption")
    pages = parse_pages(args.pages, len(reader.pages))
    writer = PdfWriter()
    for index in pages:
        writer.add_page(reader.pages[index])
    copy_metadata(reader, writer)
    atomic_pdf_write(writer, args.output, args.overwrite)


def command_rotate(args) -> None:
    if args.degrees % 90:
        raise ValueError("rotation must be a multiple of 90 degrees")
    require_input(args.input)
    reader = PdfReader(args.input)
    if reader.is_encrypted:
        raise ValueError("encrypted input is not supported without explicit decryption")
    selected = set(parse_pages(args.pages, len(reader.pages)))
    writer = PdfWriter()
    for index, page in enumerate(reader.pages):
        if index in selected:
            page.rotate(args.degrees)
        writer.add_page(page)
    copy_metadata(reader, writer)
    atomic_pdf_write(writer, args.output, args.overwrite)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect_parser = subparsers.add_parser("inspect")
    inspect_parser.add_argument("input", type=Path)
    inspect_parser.set_defaults(func=command_inspect)

    extract_parser = subparsers.add_parser("extract")
    extract_parser.add_argument("input", type=Path)
    extract_parser.add_argument("--pages")
    extract_parser.add_argument("--output", type=Path)
    extract_parser.add_argument("--overwrite", action="store_true")
    extract_parser.set_defaults(func=command_extract)

    render_parser = subparsers.add_parser("render")
    render_parser.add_argument("input", type=Path)
    render_parser.add_argument("output_dir", type=Path)
    render_parser.add_argument("--pages")
    render_parser.add_argument("--dpi", type=int, default=144)
    render_parser.add_argument("--overwrite", action="store_true")
    render_parser.set_defaults(func=command_render)

    merge_parser = subparsers.add_parser("merge")
    merge_parser.add_argument("output", type=Path)
    merge_parser.add_argument("inputs", nargs="+", type=Path)
    merge_parser.add_argument("--overwrite", action="store_true")
    merge_parser.set_defaults(func=command_merge)

    pages_parser = subparsers.add_parser("extract-pages")
    pages_parser.add_argument("input", type=Path)
    pages_parser.add_argument("output", type=Path)
    pages_parser.add_argument("--pages", required=True)
    pages_parser.add_argument("--overwrite", action="store_true")
    pages_parser.set_defaults(func=command_extract_pages)

    rotate_parser = subparsers.add_parser("rotate")
    rotate_parser.add_argument("input", type=Path)
    rotate_parser.add_argument("output", type=Path)
    rotate_parser.add_argument("--pages", required=True)
    rotate_parser.add_argument("--degrees", required=True, type=int)
    rotate_parser.add_argument("--overwrite", action="store_true")
    rotate_parser.set_defaults(func=command_rotate)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        args.func(args)
    except (ValueError, OSError, pymupdf.FileDataError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

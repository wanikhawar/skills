---
name: json-canvas
description: Create, edit, or validate Obsidian `.canvas` files when the user explicitly requests a Canvas, spatial knowledge map, or direct Canvas-file change. Handle nodes, groups, edges, layout, and vault links; use Mermaid or Markdown instead for ordinary flowcharts that do not need an explorable spatial canvas.
metadata:
  short-description: Obsidian JSON Canvas authoring and validation
---

# JSON Canvas

Use the open [JSON Canvas 1.0 specification](https://jsoncanvas.org/spec/1.0/) while preserving Obsidian-specific vault conventions. A Canvas shown as context is not permission to edit it.

Read [references/EXAMPLES.md](references/EXAMPLES.md) only when a complete layout example is useful.

## Choose Canvas deliberately

Use Canvas for spatial exploration, movable clusters, relationship maps, research boards, or user-requested visual navigation. Prefer:

- Mermaid inside a note for a compact process, dependency, or sequence;
- SVG for exact engineering geometry;
- a Markdown table for repeated-field comparison;
- a Base for property-driven database views.

## Data model

```json
{
  "nodes": [],
  "edges": []
}
```

Every node requires `id`, `type`, `x`, `y`, `width`, and `height`. Node types are `text`, `file`, `link`, and `group`. Every edge requires `id`, `fromNode`, and `toNode`.

The specification requires IDs to be unique strings. Use random lowercase 16-character hexadecimal IDs as this vault's stable convention, but do not reject an existing Canvas merely because its unique IDs use another format.

Array order is z-order: earlier nodes render below later nodes. Put groups before the nodes visually contained by them.

## Vault-specific note cards

For links to Markdown notes, use a text node whose content is solely the raw wikilink:

```json
{
  "id": "6f0ad84f44ce9c17",
  "type": "text",
  "x": 0,
  "y": 0,
  "width": 320,
  "height": 90,
  "text": "[[Entropy Generation]]"
}
```

Do not use `type: "file"` for Markdown note cards and do not add heading markers or preview prose. File nodes remain appropriate for PDFs, images, audio, and other attachments.

JSON Canvas has no standard text-alignment property. Keep the raw wikilink portable; do not claim it is centered unless the active theme/CSS was verified to center it.

## Node-specific fields

| Type | Additional fields |
| --- | --- |
| `text` | `"text": "Markdown text"` |
| `file` | `"file": "Attachments/source.pdf"`, optional `"subpath": "#page=4"` |
| `link` | `"url": "https://example.com"` |
| `group` | Optional `label`, `background`, and `backgroundStyle` (`cover`, `ratio`, or `repeat`) |

File paths must be vault-relative and use forward slashes. `subpath` begins with `#`. Preserve unknown fields added by Obsidian or plugins when editing an existing file.

## Edges

```json
{
  "id": "0123456789abcdef",
  "fromNode": "6f0ad84f44ce9c17",
  "fromSide": "right",
  "fromEnd": "none",
  "toNode": "a1b2c3d4e5f67890",
  "toSide": "left",
  "toEnd": "arrow",
  "label": "implies",
  "color": "5"
}
```

Valid sides are `top`, `right`, `bottom`, and `left`; valid ends are `none` and `arrow`. Omitted `fromEnd` defaults to `none`; omitted `toEnd` defaults to `arrow`.

Use labels only when direction alone does not communicate the relation. Do not rely on color alone.

## Layout

- Use generous spacing and a clear reading direction.
- Derive card size from content; do not shrink text to solve crowding.
- Keep 60–100 px padding inside groups and wider gaps between groups.
- Avoid node overlap except deliberate containment within groups.
- Align related nodes to a consistent grid while allowing negative coordinates.
- Keep the single most important relationship visually dominant.

For large maps, place nodes by logical layers or clusters before adding edges. Re-layout only the affected region when editing an existing Canvas unless the user requests a full redesign.

## Safe workflow

1. Read and parse the existing Canvas when present.
2. Build a semantic inventory of nodes, groups, and intended relations.
3. Preserve all existing IDs for unchanged objects.
4. Generate collision-free IDs only for new objects.
5. Position additions without disrupting unrelated regions.
6. Preserve unknown keys and established colors/layout conventions.
7. Write valid UTF-8 JSON and run `scripts/validate_canvas.py`.
8. Open or screenshot the Canvas in Obsidian when visual correctness matters. Syntax validation alone cannot prove legibility.

## Validation

```bash
python3 scripts/validate_canvas.py path/to/map.canvas
python3 scripts/validate_canvas.py --strict-layout path/to/map.canvas
```

The validator checks JSON, required fields and types, global ID uniqueness, edge references, enum values, vault-relative file paths, group z-order, Markdown-note file nodes, and optional overlap warnings. It preserves forward compatibility by warning rather than failing on unknown node types or extension fields.

---
name: engineering-visualizer
description: Design minimal, technically accurate Obsidian-native visuals for mechanical-engineering explanations and notes, including free-body diagrams, mechanisms, system boundaries, process paths, profiles, plots, dependency maps, and flows. Use only when spatial or relational structure is materially clearer as a visual; do not use for decorative images or when prose, an equation, or a table is clearer.
---

# Engineering Visualizer

Make one visual communicate one technical claim. A visual earns its place only when it exposes geometry, direction, sequence, dependency, system boundaries, or another relationship that is harder to perceive in prose.

Do not create a decorative illustration that merely repeats the surrounding sentence. Reduce the brief to the fewest elements that carry the claim; if removing an element leaves the claim intact, remove it.

For vault work, follow applicable `AGENTS.md` instructions and `vault-operator` when installed. Standalone fallback: edit only within the user’s requested scope, preserve unrelated content and conventions, then read back changes and inspect the diff.

## Choose the representation

Use the simplest editable, version-control-friendly form that preserves the needed meaning:

| Need | Preferred form |
|---|---|
| Dependencies, process flow, hierarchy, sequence, state transitions | Mermaid in the Markdown note |
| Exact geometry, mechanism layout, free-body diagram, section, vector construction | SVG attachment |
| Quantitative curve, field profile, or data comparison | Reproducible plot exported as SVG or PNG |
| Spatial knowledge map intended for exploration | Obsidian JSON Canvas, only when the user asks for a canvas |
| Exact repeated-field comparison without spatial meaning | Markdown table instead of a diagram |

Use AI image generation only for explicitly illustrative or artistic imagery, never as the source of truth for a quantitative plot, free-body diagram, mechanism, process path, or dimensioned engineering figure.

## Define the claim before drawing

Write a compact semantic brief containing:

- the single relationship the visual must establish;
- the essential objects, states, or regions;
- the required connections, directions, and labels;
- the governing convention or idealization that affects interpretation;
- what must not be implied.

Prefer roughly five to seven carrying elements. Split genuinely different claims into separate visuals rather than shrinking labels or cramming the canvas.

## Engineering correctness contract

Include what is relevant to the figure:

- coordinate axes and positive directions;
- force, moment, velocity, heat, work, mass-flow, or process arrows with unambiguous direction;
- system or control-volume boundaries, interfaces, inlets, and outlets;
- state-point labels and path direction on property diagrams;
- support types, constraints, joints, loads, and dimensions on mechanical schematics;
- variable names and units on quantitative axes;
- reference state, datum, orientation, or sign convention;
- idealizations such as not to scale, small angle, symmetry, plane stress, steady flow, or negligible losses.

Do not draw qualitative curves so precisely that they imply invented data. Do not omit a label when its absence makes two physically different readings possible. Keep visual styling subordinate to semantics: distinguish categories consistently, but never rely on color alone.

## Obsidian rendering and accessibility

- Prefer transparent backgrounds and theme-safe foregrounds so SVGs remain legible in light and dark themes.
- Use `currentColor` or a small tested palette where practical; do not encode meaning through color alone.
- Keep text as selectable `<text>` in SVG unless a verified font problem requires paths.
- Include a `viewBox`, descriptive `<title>`, and adequate margins; avoid fixed canvas dimensions that clip on mobile.
- Use notation that Obsidian's renderer can display. Mermaid labels are not a dependable substitute for MathJax-heavy equations.
- Mermaid nodes marked `internal-link` can be clickable, but they do not create Graph-view backlinks. Add an ordinary wikilink nearby when discoverability matters.

## Author and verify

1. Check the semantic brief against the governing physics before drawing.
2. Author the visual in the selected source format. Keep text concise and use notation consistent with the surrounding note.
3. Validate syntax and render when tooling permits. Inspect the rendered result, not only the source: confirm arrow direction, labels, clipping, legibility, topology, and visual hierarchy.
4. Re-check the rendered visual against the semantic brief and the equations or prose it supports. A syntactically valid figure can still be physically false.
5. If rendering is unavailable, do not claim visual verification. Keep the source editable, perform the semantic and syntax checks that are possible, and report the unverified render explicitly.

For an SVG, use an available renderer such as `rsvg-convert` or Inkscape and inspect the rendered image. Generate and inspect plots using temporary source files outside the vault. Keep those files through verification. Save plotting code or data persistently when the user requests reproducible/editable source or an established deliverable convention includes it; saving source alongside an exported figure is then part of that authorized work. Otherwise deliver the requested visual and describe its generation method in chat, without adding source files to the vault. An SVG itself is editable source.

For Mermaid embedded in a note, verify the fenced block and preview it through Obsidian when practical. For SVG or a plotted attachment, use a unique descriptive filename, apply the source-retention rule above, inspect the exported asset, and embed it by an unambiguous vault-relative path.

## Place it in Obsidian

Creating a visual in chat does not authorize writing it to the vault. Save or embed it in the vault when the user requests that visual as an attachment or as part of a note/canvas change. A request to add a figure includes its necessary exported attachment and embed; unrelated assets and persistent plotting source follow their own scope.

For an authorized edit:

- use `obsidian-markdown` or `json-canvas` when installed for the selected output format;
- follow the target note's existing attachment convention, or inspect nearby notes and Obsidian settings before choosing a new location;
- introduce the figure with one sentence stating what to notice;
- let the figure carry the spatial or relational information instead of narrating every label again;
- check the embed target and apply the shared vault validation contract.

When adapting a supplied figure without adding the image itself, extract its technically meaningful labels, directions, regions, comparisons, table entries, and caption qualifications into the replacement visual or surrounding explanation.

---
name: vault-operator
description: Operate safely on Khawar's Obsidian vault by resolving vault roots and paths exactly, preserving user conventions, limiting context and edits, and validating every authorized change. Use for vault reads, searches, note or attachment edits, moves, link checks, MOCs, and folder audits; do not infer permission to edit from a linked or active note.
license: MIT
metadata:
  short-description: Safe path-first Obsidian vault operations
---

# Vault Operator

Use this as the collection’s shared operating policy for vault work. Applicable `AGENTS.md` instructions and explicit user instructions govern; the rules below provide the portable fallback when this skill is installed elsewhere. Specialized skills should reference this policy and retain only a short standalone fallback plus their domain-specific requirements. Load a specialized format or domain skill only when the task needs it.

## Authorization boundary

- A question, selection, active note, link, attachment, image, or supplied source is context only.
- Read, search, diagnose, and answer in chat by default.
- Create, edit, move, rename, or delete a vault file only when the user explicitly requests that change.
- Permission to edit one target does not authorize changes to related notes, MOCs, properties, attachments, or settings.
- Never commit, restore, reset, or discard unrelated work unless explicitly asked.

## Resolve the vault and target

1. For an absolute path, use it exactly.
2. For a slash-containing vault-relative path, locate the vault root and join it once. A vault root contains `.obsidian/`.
3. If the target is Markdown and `.md` is omitted, try that extension before searching.
4. Treat spaces as ordinary path characters and infer the widest plausible path span from the user's sentence.
5. A URL is not a vault path. Do not misclassify `https://...` as one.
6. For a title without a path, search filenames first. For a concept, search contents.
7. Search only after exact resolution fails; then check extension, parent, case, spelling, and nearby basenames.
8. If multiple real matches remain and choosing one would change the result, ask the user.

When the Obsidian CLI is available, verify its target with `obsidian vault info=path` (an Obsidian CLI command, distinct from the Rust `vault` helper). Never rely silently on the last-focused vault.

## Tool selection

Choose the mechanism that preserves the most Obsidian semantics with the least risk:

- Exact read or inspection: direct filesystem read.
- Ranked note discovery by title, alias, or literal content: use `vault find "query"`; use `vault recent` to resume recent work. Use `rg --files` / `rg` for exact filename or scoped pattern searches.
- Small text edit: anchored patch; read the changed region back.
- Typed property change, backlink query, unresolved-link check, or Base query: Obsidian CLI when available.
- Move or rename: prefer Obsidian CLI so link updates follow vault settings.
- Markdown validation within a vault: use `vault check --quiet -- "path/to/note.md"` with the explicitly edited files. For a requested session-wide check, use `vault check --changed --quiet`. These reuse `vault-check` and check embedded-file targets. Canvas/Base validation and preservation comparisons retain their specialized helpers.
- Git inspection: run it against the repository that actually owns the target file; this vault contains nested repositories.

Do not use a GUI launcher as if it were the Obsidian CLI. Follow the `obsidian-cli` skill's capability check.

## Fast vault commands

The Rust `vault` and `vault-check` binaries are installed together in `~/.local/bin`. If `vault` is unavailable, build and install both from `../obsidian-markdown/scripts/vault-check/` relative to this skill; build instructions are in `obsidian-markdown`. Run the installed binary directly during routine work.

```bash
vault find "entropy" --limit 10
vault find "entropy" --paths
vault recent --limit 10
vault check --quiet -- "Thermodynamics/Basic Thermodynamics/Entropy.md"
vault check --changed --quiet
```

The root is selected by `--root PATH` before the command, then `VAULT_ROOT`, then the nearest ancestor containing `.obsidian/`. Prefer an explicit root when working outside the target vault or when an environment override could select another vault. Explicit check filenames remain relative to the working directory. Quote queries and paths.

`find` ranks exact titles, exact aliases, partial titles, aliases, paths, then content; matching is case-insensitive and requires all query terms. `recent` uses filesystem modification time, which syncing can change. Discovery skips hidden directories, symlinks and `node_modules`; it does not build a persistent index. `--paths` prints vault-relative paths. `--uri N` prints the Obsidian URI for the Nth displayed result; `--open N` opens it via `xdg-open` only when opening a note is requested. Neither edits note content.

`check --changed` includes staged, unstaged and untracked Markdown in the vault's Git repository, excluding deleted files and hidden paths. It does not recurse into nested Git repositories. For a focused edit, pass explicit files so unrelated user changes are not treated as task findings. Exit 0 means no errors; 1 means validation/embed findings (or no search matches); 2 means a usage/operational error. Warnings still require judgment.

Embedded-file checking covers wiki embeds and ordinary inline Markdown images; it reports missing or ambiguous file targets. It does not validate heading/block fragments, reference-style/HTML images, complex Markdown destinations, or plugin semantics. Use Obsidian for exact resolution when needed; a static pass does not prove rendering or semantic correctness. No command rewrites notes.

## Context and placement

For one note, read the target and only the local context necessary to interpret the requested change. Do not turn a focused edit into a vault-wide audit.

Before creating a note:

1. Check the intended path.
2. Search equivalent and near-equivalent basenames.
3. Check whether the material already has an authoritative section elsewhere.
4. Create the smallest correct home only when it is genuinely missing or explicitly requested.

Use the local folder's conventions for frontmatter, tags, headings, attachments, link style, and MOC placement. Do not impose a universal template.

## Editing contract

- Preserve frontmatter, aliases, tags, wikilinks, equations, citations, block IDs, and unique technical nuance unless the request changes them.
- Use `$...$` and `$$...$$` for LaTeX; do not introduce `\[...\]`.
- Avoid assistant commentary in study notes.
- Do not create raw MCQ replicas in study notes. Convert them to declarative exam insights and retain exam/year provenance when useful.
- Treat unrelated dirty files as user-owned.

When explicitly asked to incorporate supplied material, inventory every distinct correct definition, statement, equation, condition, comparison, qualification, and meaningful figure/table/caption detail. Consolidate genuine duplicates without losing additional nuance. Correct or omit only demonstrably wrong, unverifiable, inconsistent, or out-of-scope material.

For incorporation or reorganization, retain a pre-edit snapshot of the current note in a unique temporary file outside the vault. When `obsidian-markdown` is installed, use its Rust `vault-check preserve BEFORE AFTER` after editing instead of ad hoc Python structural comparisons. Review findings against the authorized changes; this supplements the completeness inventory and diff review, without proving semantic preservation.

## Validation

After an authorized change, verify what applies:

- Read every changed file back.
- Run the Markdown, Base, Canvas, SVG, or PDF validator appropriate to the format.
- Check referenced files and embeds exist; use `obsidian unresolved` when the CLI is available.
- Confirm no duplicate note or unintended file was created.
- Inspect the owning repository's diff and run `git diff --check` for meaningful text edits.
- Preview or render when syntax validation cannot establish visual correctness.

For incorporation edits, report separately:

- what was added or expanded;
- what was already present and consolidated;
- what was corrected or omitted and why, explicitly saying when nothing was omitted.

For other vault changes, report the exact changed paths, the outcome, validation performed, and any unresolved issue.

## Reusable improvements

Only when a task reveals a durable new vault rule, ask:

> I found a reusable vault rule: `<rule>`. Should I add it to `<skill>` so future agents follow it?

Do not ask for one-off facts or temporary workflow details.

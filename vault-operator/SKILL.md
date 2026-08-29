---
name: vault-operator
description: "Use when operating on Khawar's Obsidian vault: resolve paths exactly, route vault tasks, edit safely, validate notes, and ask before adding reusable workflow rules."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [macos]
metadata:
  hermes:
    tags: [obsidian, vault, note-taking, path-handling, workflow]
    related_skills: [obsidian]
---

# Vault Operator

## Purpose

Base operating protocol for Khawar's Obsidian vault. Use this before any vault read, edit, audit, creation, link check, MOC update, or study-note maintenance task.

Primary rule: **path-first, search-second**. Spaces in file or folder names are normal path characters.

## Load With

Also load specialized skills when the task needs them:

- `obsidian` for filesystem-first note operations.
- A syllabus/PYQ skill for broad exam coverage checks.
- A formula/audit skill when the task is formula-heavy or subject-specific.
- Anki/OCR/dashboard skills only when the user asks for those workflows.

Do not expand scope just because extra skills exist.

## Task Intent Router

Classify the user request before acting:

| Intent | Signal | First action |
|---|---|---|
| Exact path | Contains `/` or is absolute | Direct-read/direct-use path |
| Note title | Short title, no `/` | Filename search |
| Topic/content | Concept phrase | Content search |
| Folder scope | Folder or subject audit | List folder/MOC |
| Creation | “create/add new note” | Check duplicates first |
| Rewrite/audit | “audit/fix/improve/clearer” | Read target + minimal context |
| External workflow | Anki/OCR/web/dashboard | Load matching skill |

## Path Handling Contract

1. If the user provides a slash-containing path, treat it as a path, not a search query.
2. If the path is absolute, use it exactly. Do not prepend the vault root.
3. If the path is vault-relative, join it to the vault root once.
4. If a Markdown note path lacks `.md`, try the same path with `.md` first.
5. Never split paths on spaces. Never ask for clarification only because a folder or file name contains spaces.
6. Search only after direct path resolution fails, or when the user gave a title/topic rather than a path.
7. If direct resolution fails, check extension, parent folder, case/spelling, and nearby filename matches before asking the user.

## Path Confidence Levels

- **A: Exact path** — absolute path or slash-containing vault-relative path. Direct-use first.
- **B: Likely note title** — no slash, likely filename. Search filenames first.
- **C: Topic phrase** — conceptual phrase. Search contents.
- **D: Ambiguous shorthand** — could refer to multiple folders or concepts. Resolve from context; ask only if materially ambiguous.

## File Tool Rules

Prefer file tools over shell commands:

- Read exact note: `read_file`.
- Find files: `search_files(target="files")`.
- Search note text: `search_files(target="content", file_glob="*.md")`.
- Small anchored edit: `patch`.
- Whole-note rewrite: `write_file`.
- Multi-file checks or fragile LaTeX edits: `execute_code` with Python file I/O.
- Git status/diff/checks: `terminal`.

Avoid shell commands for normal note reads/searches. If terminal is unavoidable, quote every path or use Python `pathlib`; never concatenate unquoted paths.

## Context-Minimum Protocol

Do not turn a small task into a vault-wide audit.

For a single-note task:

1. Resolve target path directly when provided.
2. Read the target note.
3. Read only necessary local context: MOC, formula sheet, or closely related notes.
4. Edit only what the user asked for.
5. Verify the changed file.

For folder-wide tasks, list notes and read in batches. For topic discovery, search filenames before contents.

## Duplicate Prevention

Before creating a note:

1. Check exact intended path.
2. Search for same or near-equivalent basename.
3. Check whether the topic already exists as a section in a broader note.
4. Create a new note only when it is genuinely missing or explicitly requested.

Do not create duplicates due to path-resolution failure.

## Content Placement Rules

Choose the smallest correct home for new content:

- Navigation/index content → MOC or dashboard.
- Detailed concept/derivation → topic or chapter note.
- Short formula reminder → formula sheet or summary section.
- Repeated exam trap → relevant topic note; optionally formula sheet if formula-related.
- Raw extracted material → keep separate from polished notes until cleaned.
- Flashcards → Anki workflow, not the note body unless requested.

## Editing Rules

- Preserve frontmatter, aliases, tags, wikilinks, equations, and unique domain nuance.
- Use `$...$` and `$$...$$` for LaTeX; do not use bracket-style display math.
- Do not leave assistant meta-commentary in study notes.
- Prefer patch for small edits; rewrite only when structure/clarity requires it or the user asks.
- For LaTeX-heavy edits, avoid transformations that may corrupt braces; verify by reading back.

## Validation Checklist

Before final reply after editing, verify what is relevant:

- Exact user path was attempted before search when a path was provided.
- File was read back after modification.
- `$$` delimiters are balanced.
- No `\[...\]` display math was introduced.
- Inline `$` usage is not obviously unbalanced.
- Markdown tables have consistent pipe counts.
- Wikilinks resolve where practical.
- `git diff --check` passes for meaningful edits.
- No duplicate note was created.

## Output Contract

Final responses after vault work should be brief and concrete:

- changed path(s),
- what changed,
- what was verified,
- any unresolved ambiguity or recommended next step.

Do not report unnecessary process detail.

## Error Recovery

If a path read fails:

1. Try `.md` if omitted.
2. Check whether the parent folder exists.
3. Search within the intended parent if available.
4. Search by basename if needed.
5. Ask the user only after tool-based resolution fails or multiple real matches remain.

If an edit corrupts content, stop, read the file, repair or restore from git only with user-safe scope. Never discard unrelated changes.

## Git Safety

- Check git status/diff for large or risky edits when practical.
- Never commit, reset, restore, or delete unless explicitly asked.
- Treat unrelated dirty files as user-owned.
- Use `git diff --check` as a whitespace/syntax sanity check after edits.

## Self-Improvement Hook

At the end of a vault task, ask whether to update a skill only when the task revealed a reusable rule, recurring annoyance, folder convention, validation check, routing rule, or agent failure mode.

Ask in this form:

> I found a reusable vault rule: `<rule>`. Should I add it to `<skill>` so future agents follow it?

Do not ask for one-off task progress, temporary details, or facts likely to become stale soon.

Place new rules in the right skill:

- Path handling, routing, vault safety → this skill.
- Subject-note audit workflow → audit skill.
- Formula conventions → formula skill.
- Syllabus/PYQ coverage → syllabus/PYQ skill.
- Anki cards → Anki skill.
- OCR/PDF ingestion → OCR/document skill.
- MOC/dashboard rules → MOC/dashboard skill.

## Non-Hermes Agents

If using another agent system, preserve the same behavior: direct filesystem access for exact paths, preserve spaces exactly, quote shell paths, search only when the target is not a path or direct resolution fails, and verify Markdown after edits.

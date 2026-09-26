---
name: semantic-note-linker
description: Find meaningful connections between Obsidian notes and suggest or insert Wikipedia-style inline wikilinks to verified notes or sections. Use for requests to interlink notes, connect related concepts, or add internal links across a note, folder, or vault. Preserve all original information and require Git diff review for edits; do not use for ordinary concept questions or automatically edit notes merely because they are attached.
---

# Semantic Note Linker

Create useful reading paths, not a dense keyword index. Use semantic judgment to discover relationships and deterministic comparisons to preserve content. This is an on-demand workflow, not a background watcher.

## Scope and modes

- **Suggest:** Read and propose links without changing files. Use when the user requests suggestions, a preview, or has not authorized edits.
- **Apply:** Insert high-confidence links when explicitly requested. Authorization to “link this note” is sufficient for routine linking and validation in that note; do not ask for approval of every link.
- Separate the **write scope** (authorized notes) from the **discovery scope** (potential destinations, normally the vault unless restricted). A destination is not automatically an authorized source for edits.
- For an ambiguous write scope, ask one targeted question. Attached notes and permission to create this skill do not authorize running it against the vault.
- Follow applicable AGENTS.md and, when available, the vault-operator skill. Resolve paths absolutely for filesystem/Git operations. Exclude hidden/configuration directories, trash, dependency folders, and non-note files from ordinary discovery. Do not create missing notes, change aliases/frontmatter, rename files, repair unrelated links, or add prose without separate authorization.

## Discover and evaluate

1. Read each source note fully, including existing links. Identify meaningful concepts, their contextual sense, and useful prerequisites, definitions, applications, comparisons, or derivations.
2. Find candidates economically: titles, aliases, headings, existing link neighborhoods, then targeted content searches. Where installed, use `vault --root ABSOLUTE_VAULT_ROOT find "concept" --limit 10` for titles and aliases, and `vault-search --root ABSOLUTE_VAULT_ROOT search "concept" --sections --compact --limit 6` for sections whose content explains the concept (it tolerates misspellings and word forms but does not infer synonyms); otherwise use scoped filesystem search. Search synonyms, alternate notation in prose, and domain-specific terminology rather than relying solely on exact words. Do not upload the vault or install an embedding service.
3. Read candidate passages with enough surrounding context to verify their meaning. Names, tags, search hits, and shared equations alone are not evidence of relevance. Use Obsidian's CLI for resolved links and backlinks when these queries help, following the obsidian-cli skill and explicitly targeting the correct vault.
4. Choose the most specific substantive explanation. Prefer a concept note over a broad MOC when the reader needs an explanation, or a verified section when a broader note contains the right material. Do not assume the longest note is best. Account for duplicate titles and context-dependent terms such as stress, work, head, or efficiency.
5. Accept a link only if the source phrase and destination agree in meaning, the destination adds useful understanding, and the relationship is clear from the existing sentence. A related subject is not necessarily a valid target for that phrase. If an explanatory bridge would need to be written, suggest that separately instead of inventing text or disguising a loose relationship as a synonym.

For each accepted candidate retain a small working ledger: source path and occurrence, exact original phrase, destination path and optional heading, proposed markup, and a short evidence-based reason. Keep this in temporary working data or chat, not a new vault report unless requested. Skip uncertainty rather than manufacturing confidence scores.

## Inline linking rules

- Preserve the exact original phrase as the displayed alias: `[[exact/vault-relative/Note#Verified heading|original phrase]]`. Use a verified, unambiguous path, retaining local extension conventions. This is an illustrative syntax template, not a real target.
- Use note-level links when they are sufficient. Use heading or existing block targets only after confirming the exact heading/block exists and resolves; never create a heading/block ID merely to link to it.
- Prefer the first contextually useful occurrence. Repeat only when a distant, independently read section genuinely benefits; do not link every occurrence or impose a link quota.
- Do not add self-links or nested links, retarget existing links, or modify embeds, existing Markdown links, reference definitions, citations, URLs, frontmatter, code, LaTeX, headings, block IDs, or HTML. Link ordinary prose; skip structurally awkward phrases rather than rewriting them.
- In Markdown tables escape the wikilink alias separator as `\|` where required and validate the row structure. If an exact phrase cannot safely be represented without changing existing content, skip it.
- Do not append “Related notes” sections or package links as MCQ answers. Do not automatically add reciprocal links: backlinks already support reverse discovery. A reverse inline link requires its own useful anchor and authorization to edit that note.
- Re-running the skill must not duplicate links or keep increasing link density without a new reason.

## Mandatory preservation and Git diff gate

**Every apply run must prove that only the approved link markup was introduced. Reviewing a clean-looking diff alone is not proof of content preservation.**

### Before editing

1. Determine the owning Git repository of each source; the vault can contain nested repositories. Capture scoped `git status --short`, unstaged `git diff -- PATH`, and staged `git diff --cached -- PATH` baselines using `git -C ABSOLUTE_REPOSITORY` and exact repository-relative paths after `--`.
2. Save a byte-for-byte snapshot of each current source in a unique temporary directory outside the vault. Record its hash. Snapshot the current working file, **not HEAD**: pre-existing uncommitted and staged work belongs to the user. Preserve encoding, line endings, whitespace, and final newline.
3. Record the exact edit operations against that snapshot, including the original spans and the link delimiters/targets to insert. Each operation must retain the original phrase verbatim and only add link syntax around it.
4. If Git is unavailable, or a source is not in a Git repository, remain in suggest mode for that source and explain that the requested repository diff safeguard cannot be met. An untracked file inside a repository is supported via snapshot comparison below; an empty ordinary `git diff` does not verify it.

### Apply narrowly

Recheck the current source against its snapshot immediately before writing. If it has changed, re-read and rebuild that file's plan instead of overwriting concurrent edits. Apply exact, localized edits only; never run blind global keyword replacement or reserialize the whole note through a Markdown formatter.

### Compare and inspect before reporting success

For **every** changed source:

1. Read it back and run `git diff --no-index -- BEFORE_SNAPSHOT ABSOLUTE_CURRENT_FILE`. Both paths must be absolute. Exit 1 means differences, not a command failure; higher exit codes require investigation. Inspect every hunk. This isolates this run, including for untracked files and dirty working trees.
2. Also inspect the owning repository's scoped `git diff -- PATH` and `git diff --cached -- PATH` against the captured baselines. The index must be unchanged; distinguish existing user changes from this run. Do not stage, commit, restore, reset, or discard work.
3. Deterministically reconstruct the original bytes by removing **only this run's recorded added link markup**, preserving each original phrase. Require exact byte equality with the pre-edit snapshot. Do not strip all wikilinks, normalize whitespace, compare only word counts, or rely on a semantic summary. Alternatively construct the expected post-edit bytes from the snapshot and recorded insertion-only operations and require exact equality with the actual result. Use a short local verification script if needed.
4. Any removed/reworded text, lost equation, changed punctuation, modified existing link, or unrelated structural change fails the gate—even if judged redundant or technically incorrect. Linking is not a consolidation or correction task. Fix only this run's unintended changes and repeat the comparison; if concurrent changes prevent safe correction, stop the affected file and report it rather than overwriting user work.
5. Run scoped `git diff --check`; for an untracked file also use `git diff --no-index --check -- BEFORE_SNAPSHOT ABSOLUTE_CURRENT_FILE`. Attribute pre-existing findings separately. Validate affected Markdown/table syntax and target files/headings. When installed, use the vault's Markdown checker on the explicit edited paths; it is supplementary, not a substitute for link resolution or preservation proof.
6. Use Obsidian's resolver for live semantic validation where available. If unavailable, verify exact paths and headings statically, skip ambiguous targets, and explicitly label this limitation. Do not claim a rendered preview or live resolution unless performed.

Do not report successful preservation until both the diff inspection and exact insertion-only comparison pass. Retain temporary snapshots until verification is complete.

## Output

In suggest mode, show concise proposals with source phrase, destination, and why the link helps. Separate ambiguous candidates that need a choice.

In apply mode, report the changed notes using Obsidian wikilinks, the number of links added, and any important skipped cases. State whether Git diff review and exact content-preservation checks passed, and distinguish static from live validation. When verified, explicitly state: **No original information was deleted or rewritten; only inline link markup was added.** Report failures or limitations honestly. Do not claim whole-vault coverage after a partial run.

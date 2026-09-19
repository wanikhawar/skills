# Vault search

A local Rust search utility for this Obsidian vault. SQLite FTS5 indexes file paths, note titles, YAML aliases/tags, Markdown headings, and text. Search combines weighted full-text ranking, English stemming, prefix matching, and spelling correction. It returns exact paths, section line ranges, excerpts, suggested wikilinks, and Obsidian URIs so an assistant can retrieve a source and read it before answering.

## Everyday use

Run inside the vault or set `--root "/absolute/path/to/vault"`. An explicit root takes precedence over `VAULT_ROOT`, which takes precedence over the nearest ancestor containing `.obsidian/`.

```bash
vault-search search "why does a flywheel not regulate mean speed"
vault-search search "flyweel" --compact --limit 3
vault-search search "fluctuation of energy" --sections
vault-search read "Theory of Machines/Flywheel.md" --line 1 --lines 100
vault-search open "Theory of Machines/Flywheel.md"
vault-search open "Theory of Machines/Flywheel.md" --uri
```

`search` refreshes the index automatically before every query. New notes become searchable on the next search, edits replace old indexed content, and deleted/renamed files are removed. No background process or manual reindexing is required. The refresh scans file metadata and reads only files whose size or nanosecond modification time changed. `read` reads the current source file, not a cached excerpt. A file changed after a search may have different line numbers; search again when necessary.

By default, search returns three files, showing the best matching section of each. Start with `--compact` for concise JSON (relative path, heading, line range, snippet, content-indexed flag, and spelling expansions); use `--limit 6` when more candidates are needed. `--compact` takes precedence over `--json`. Full `--json` remains available for diagnostics and includes refresh counts, absolute paths, wikilinks, URIs, and scores.

Terms normally match with OR for broad discovery. `--all-terms` requires each non-stopword term (or its stem/prefix/spelling alternative) to match within a single indexed section, including its path/title/alias metadata. It does not require an exact phrase or matches across different sections. Query text is not Boolean syntax: operators and negation are not interpreted. Literal multiword phrases receive a ranking boost; heading-only and bare-wikilink sections receive less weight. YAML frontmatter does not count as explanatory prose, and close whole-title spelling matches are preferred over unrelated typo expansions. These are ranking heuristics, not proof of conceptual coverage. `--sections` includes multiple sections per file. `--no-fuzzy` disables typo expansion while keeping stemming and prefix matching. Use `--json` for structured output, including refresh counts and spelling expansions. Exit codes: 0 success, 1 no search results, 2 operational/usage error.

To refresh manually or reread everything (for example, after a tool preserves both timestamps and file sizes):

```bash
vault-search index
vault-search index --rebuild
```

## Indexed content and limits

- Every regular, non-hidden file in the included vault tree is indexed by filename and path, including images and PDFs.
- UTF-8 Markdown and common text formats up to 4 MiB also have their content indexed. Larger files, unrecognized formats, invalid UTF-8, and binary data receive filename/path entries only. PDF extraction and image OCR are not included.
- Markdown ATX headings (`#` through `######`) divide notes into sections with original line numbers. Code-fence headings are ignored. Setext headings remain searchable as text but do not define sections. YAML aliases, alias, and tags receive additional ranking weight.
- Hidden files/folders, symlinks, `API KEYS`, `node_modules`, and `target` are excluded. Hidden Obsidian/plugin data and the search utility's own index are therefore outside the search corpus. Git-ignored subject notes in otherwise included folders remain searchable.
- Nothing is uploaded. The index contains copies of indexed text and lives at `.agents/skills/vault-operator/scripts/vault-search/index.sqlite3`. Git ignores the index and build artifacts; a separate file-sync application may still sync them under its own rules.
- This is lexical retrieval, not an embedding model: it tolerates misspellings and word endings but cannot infer arbitrary synonyms. Assistants should try related terms and verify source text. Search considers at most 24 distinct query terms, 6 spelling alternatives per term, and the top 1,000 full-text section candidates.
- A failed traversal or file read aborts the refresh instead of silently discarding indexed sources. Concurrent commands wait up to 30 seconds for the database. Paths with non-UTF-8 filenames are unsupported.

## Build and install

Requires Rust/Cargo and SQLite development libraries with FTS5 support. From the vault root:

```bash
cargo test --manifest-path .agents/skills/vault-operator/scripts/vault-search/Cargo.toml
cargo build --release --manifest-path .agents/skills/vault-operator/scripts/vault-search/Cargo.toml
cargo install --path .agents/skills/vault-operator/scripts/vault-search --root "$HOME/.local" --locked
vault-search index
```

Ensure `$HOME/.local/bin` is on `PATH`. Alternatively run the built binary at `.agents/skills/vault-operator/scripts/vault-search/target/release/vault-search`, or use `cargo run --release --manifest-path .agents/skills/vault-operator/scripts/vault-search/Cargo.toml -- search "topic"`. The `open` command uses `xdg-open` on Linux and `open` on macOS; `open --uri` prints the encoded URI without launching an application.

The root `AGENTS.md` instructs assistants to search this vault before answering subject questions, read the relevant sections, and distinguish note-supported information from additional reasoning. Indexing and searching never modify source notes.

## Efficient tool routing

Use indexed search for concepts and misspellings; `vault find` or `rg --files` for known titles; `rg -n` for literal text/formulas in a known source; `vault-search read` for current evidence; `vault recent` to resume work; and `vault check` after note edits. Read the source before citing it. Try related terms when needed: spelling expansion is not semantic synonym search.

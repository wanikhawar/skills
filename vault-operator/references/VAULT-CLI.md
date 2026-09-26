# Vault CLI reference

Build, install, and behaviour details for the Rust `vault` workflow CLI and its sibling `vault-check` validator. Load this only when building/installing the binaries, choosing flags, or interpreting exit codes and limitations.

## Sources and installation

The Rust `vault` source lives in `scripts/vault-cli/` relative to the `vault-operator` skill directory; `vault-search` lives in `scripts/vault-search/` (see its [README](../scripts/vault-search/README.md) for build/install steps). The separate Markdown validator `vault-check` lives in `../obsidian-markdown/scripts/vault-check/` relative to the skill directory. Install `vault` and `vault-check` into the same directory because `vault check` invokes its sibling validator. Build or install both from their respective manifests when missing or after source changes; run the installed binaries directly during routine work. Set `skill_dir` to the absolute `vault-operator` skill directory (the one containing `SKILL.md`) before running the commands below.

```bash
cargo build --release --locked --manifest-path "$skill_dir/scripts/vault-cli/Cargo.toml"
cargo build --release --locked --manifest-path "$skill_dir/../obsidian-markdown/scripts/vault-check/Cargo.toml"
cargo install --path "$skill_dir/scripts/vault-cli" --root "$HOME/.local" --locked --force
cargo install --path "$skill_dir/../obsidian-markdown/scripts/vault-check" --root "$HOME/.local" --locked --force
```

`cargo install` produces the shared `~/.local/bin` pair; ensure it is on `PATH`. If installing is unavailable, place both built binaries in one directory before running `vault check`.

## Commands

```bash
vault find "entropy" --limit 10
vault find "entropy" --paths
vault recent --limit 10
vault check --quiet -- "Thermodynamics/Basic Thermodynamics/Entropy.md"
vault check --changed --quiet
```

The root is selected by `--root PATH` before the command, then `VAULT_ROOT`, then the nearest ancestor containing `.obsidian/`. Prefer an explicit root when working outside the target vault or when an environment override could select another vault. Explicit check filenames remain relative to the working directory. Quote queries and paths.

## Behaviour and limitations

`find` ranks exact titles, exact aliases, partial titles, aliases, paths, then content; matching is case-insensitive and requires all query terms. `recent` uses filesystem modification time, which syncing can change. Discovery skips hidden directories, symlinks and `node_modules`; it does not build a persistent index. `--paths` prints vault-relative paths. `--uri N` prints the Obsidian URI for the Nth displayed result; `--open N` opens it via `xdg-open` only when opening a note is requested. Neither edits note content.

`check --changed` includes staged, unstaged and untracked Markdown in the vault's Git repository, excluding deleted files and hidden paths. It does not recurse into nested Git repositories. For a focused edit, pass explicit files so unrelated user changes are not treated as task findings. Exit 0 means no errors; 1 means validation/embed findings (or no search matches); 2 means a usage/operational error. Warnings still require judgment.

Embedded-file checking covers wiki embeds and ordinary inline Markdown images; it reports missing or ambiguous file targets. It does not validate heading/block fragments, reference-style/HTML images, complex Markdown destinations, or plugin semantics. Use Obsidian for exact resolution when needed; a static pass does not prove rendering or semantic correctness. No command rewrites notes.

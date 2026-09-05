---
name: obsidian-cli
description: Use the registered Obsidian CLI for operations that need Obsidian semantics, including exact vault queries, typed properties, backlinks, unresolved links, Bases, moves and renames with link updates, or plugin/theme debugging. Do not use merely because a vault file can be read or patched directly, and do not mistake the desktop GUI launcher for the CLI.
metadata:
  short-description: Safe Obsidian CLI operations and debugging
---

# Obsidian CLI

Use the CLI when Obsidian's index, link resolver, typed properties, history, Base engine, or developer runtime materially improves the task. The desktop app must be installed and running; the CLI requires a current installer and must be enabled under **Settings → General → Command line interface**.

For vault work, follow applicable `AGENTS.md` instructions and `vault-operator` when installed. Standalone fallback: edit only within the user’s requested scope, preserve unrelated content and conventions, then read back changes and inspect the diff.

## Helper paths and dependencies

Resolve the directory containing this loaded `SKILL.md` to an absolute path. In the commands below, set `skill_dir` to that directory; the example value is a placeholder. Keep the working directory unchanged so relative input and output paths retain their meaning in the user’s workspace. Quote all paths.

```bash
skill_dir="/absolute/path/to/obsidian-cli"
```

The helper uses Python 3 and its standard library.

## Capability check

Before depending on the CLI:

1. Resolve the executable with `command -v obsidian`.
2. On Linux, prefer the registered CLI at `~/.local/bin/obsidian`. A distro `/usr/bin/obsidian` may be a shell wrapper that launches Electron.
3. Inspect a suspicious wrapper before running it. Do not pass `help` to a GUI launcher.
4. Run a short-timeout `version` or `help` check. If it does not return CLI output promptly, treat the CLI as unavailable and use safe filesystem operations.
5. Do not repeatedly launch Obsidian while probing availability.

The bundled check performs these tests without executing a known GUI wrapper:

```bash
python3 "$skill_dir/scripts/check_cli.py" --json
```

The check requires a recognizable version response and Obsidian help containing `read`, `search`, and `vault` commands. An unfamiliar response is unverified, even if the process exits successfully; inspect it before changing the detector. Diagnostics go to stderr in JSON mode.

Read the returned `executable` field and use that exact absolute invocation path for every subsequent command. Set `obsidian_cli` to that value (the following value is a placeholder), rather than resolving `obsidian` again through `PATH`:

```bash
obsidian_cli="/absolute/path/returned/by/check_cli"
```

Current documentation: <https://obsidian.md/help/cli>

## Target the correct vault and file

If the current working directory is inside a vault, that vault is the default; otherwise the active vault is used. For certainty, put `vault=<name-or-id>` before the command and verify:

```bash
"$obsidian_cli" vault="Academic" vault info=path
```

For files:

- `path="Folder/Exact Note.md"` is exact from the vault root and is preferred whenever a path is known.
- `file="Note Name"` resolves like a wikilink and can be ambiguous when basenames repeat.
- Without `file` or `path`, many commands target the active file; avoid that implicit target in automated work.

Quote every value containing spaces. Parameters use `name=value`; flags have no value. Use `\n` and `\t` in CLI content strings.

## High-value read-only commands

```bash
"$obsidian_cli" vault="Academic" read path="Thermodynamics/Entropy.md"
"$obsidian_cli" vault="Academic" search query="entropy generation" path="Thermodynamics" limit=20
"$obsidian_cli" vault="Academic" search:context query="entropy generation" path="Thermodynamics" limit=20
"$obsidian_cli" vault="Academic" backlinks path="Thermodynamics/Entropy.md" counts
"$obsidian_cli" vault="Academic" links path="Thermodynamics/Entropy.md"
"$obsidian_cli" vault="Academic" unresolved verbose
"$obsidian_cli" vault="Academic" properties path="Thermodynamics/Entropy.md" format=yaml
"$obsidian_cli" vault="Academic" property:read name="status" path="Thermodynamics/Entropy.md"
"$obsidian_cli" vault="Academic" base:query path="Dashboards/Review.base" view="Due" format=json
"$obsidian_cli" vault="Academic" diff path="Thermodynamics/Entropy.md"
```

Use `format=json` when another tool will parse the output. Use `total` for inexpensive counts.

## Mutations

Run mutations only when the user authorized the corresponding vault change.

- Use `create path=...` without `overwrite` first; add `overwrite` only when replacement is explicit and the target was inspected.
- Use `property:set ... type=text|list|number|checkbox|date|datetime` for typed properties.
- Prefer `move` or `rename` for existing notes so Obsidian can update internal links according to vault settings.
- `delete` uses trash by default. Never add `permanent` unless permanent deletion was explicitly requested.
- Treat `history:restore`, `sync:restore`, plugin install/uninstall, and theme changes as separate state-changing actions requiring explicit scope.

After a mutation, read the target back and run a relevant query such as `unresolved`, `properties`, or `base:query`.

## Plugin and theme development

After an authorized code change:

```bash
"$obsidian_cli" plugin:reload id=my-plugin
"$obsidian_cli" dev:errors
"$obsidian_cli" dev:console level=error
"$obsidian_cli" dev:screenshot path="/tmp/my-plugin.png"
"$obsidian_cli" dev:dom selector=".workspace-leaf" text
```

Use `eval` only with code whose scope and effects are understood. Prefer a read-only DOM or application query before mutation. Clear error buffers only when doing so will not erase evidence needed by the user.

If the CLI is unavailable, report that semantic or visual verification could not be performed; do not claim it succeeded.

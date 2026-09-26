# Agent Skills

Twelve reusable skills for Obsidian workflows, mechanical-engineering study, web content extraction, and PDF processing. Each skill provides task-specific instructions; some include reference material and executable helpers.

## Included skills

### General utilities

| Skill | What it does |
| --- | --- |
| [Defuddle](defuddle/SKILL.md) | Extracts readable Markdown and source metadata from public HTML pages, with checks for missing technical content and fallbacks for unsupported pages. |
| [PDF](pdf/SKILL.md) | Inspects, extracts, renders, combines, splits, and rotates PDFs; guides OCR, creation, and Obsidian incorporation while preserving page provenance. |

### Obsidian

| Skill | What it does |
| --- | --- |
| [JSON Canvas](json-canvas/SKILL.md) | Creates and edits spatial knowledge maps, preserving node IDs, extension fields, and local layout conventions. Includes a structural validator. |
| [Obsidian Bases](obsidian-bases/SKILL.md) | Builds property-driven views from the vault’s actual schema. Checks structure and formula references, with warnings for suspicious duration expressions. |
| [Obsidian CLI](obsidian-cli/SKILL.md) | Verifies the registered CLI and uses its exact executable path for vault queries, typed properties, link-aware operations, and debugging. |
| [Obsidian Markdown](obsidian-markdown/SKILL.md) | Authors notes with wikilinks, embeds, callouts, properties, and math. Includes static checks that distinguish inline code from math and link syntax. |
| [Semantic Note Linker](semantic-note-linker/SKILL.md) | Finds meaningful connections between notes and suggests or inserts verified inline wikilinks, preserving original content and gating edits behind a Git diff review. |
| [Vault Operator](vault-operator/SKILL.md) | Defines shared vault rules for exact path resolution, scoped edits, content preservation, and validation. Contains author-specific conventions to review before reuse. |

### Mechanical engineering

| Skill | What it does |
| --- | --- |
| [Engineering Visualizer](engineering-visualizer/SKILL.md) | Creates technically accurate Mermaid diagrams, SVG schematics, and plots. Distinguishes temporary generation files from persistent source deliverables. |
| [Feynman Technique](feynman-technique/SKILL.md) | Runs interactive learn-by-explaining sessions: the learner explains first, then targeted probes, gap repair, and a transfer problem, grounded in the learner's notes. |
| [ME Concept Tutor](me-concept-tutor/SKILL.md) | Explains concepts, derivations, and exam distinctions from governing principles, with depth matched to the request. |
| [Syllabus Gap Audit](syllabus-gap-audit/SKILL.md) | Compares notes with official syllabi, assessing coverage, correctness, and navigation independently. Separates stated syllabus depth from evidence-backed preparation inferences. |

## Installation

Clone the repository:

```bash
git clone https://github.com/wanikhawar/skills.git
```

Copy the skill directories you need into the location supported by your agent. Keep each directory intact so its `SKILL.md`, `references/`, and `scripts/` remain together. Keep the repository’s `tests/` directory if you want to run the collection’s regression tests.

For clients that support project-scoped `.agents/skills` discovery:

```text
your-project/
└── .agents/
    └── skills/
        ├── vault-operator/
        │   └── SKILL.md
        ├── obsidian-markdown/
        │   ├── SKILL.md
        │   ├── references/
        │   └── scripts/
        └── engineering-visualizer/
            └── SKILL.md
```

Discovery paths vary by agent client. Check your client’s configuration if skills are not loaded automatically.

For vault work, include `vault-operator` alongside the relevant format or domain skills. Specialized skills also contain compact fallbacks for separate installation. Review personal conventions, especially Canvas note cards and vault editing rules, before using the collection in another vault.

## Usage

Ask for the task in ordinary language. A compatible agent uses each skill’s frontmatter description to select relevant instructions. You can also name a skill explicitly.

Examples:

```text
Extract this public article as Markdown, retaining its source metadata.

Explain why entropy is a property from first principles.

Add a free-body diagram to this note and save the SVG attachment.

Create an Obsidian Base listing unfinished thermodynamics notes.

Audit my Heat Transfer notes against the current GATE and ESE syllabi.

Fill the high-priority gaps identified in this audit.

Merge these PDFs in the supplied order and verify the page boundaries.

Link this heat-transfer note to the relevant concept notes already in the vault.
```

Questions, linked notes, selections, and supplied sources provide context; they do not authorize note edits. Ask explicitly to create, update, save, or incorporate material when you want a vault change. An audit produces a report by default; filling gaps is a separate requested mode.

For requested incorporation, the shared policy preserves every distinct correct in-scope source point, consolidates duplicates without losing qualifications, and reports additions, consolidation, and omissions or corrections. Explicit requests for summaries or selected material determine the scope.

## Shared policy and deliverables

[Vault Operator](vault-operator/SKILL.md) holds the collection’s shared operating rules, subject to applicable `AGENTS.md` and user instructions. Specialized skills add format-specific checks and short standalone fallbacks.

- Resolve targets exactly, including paths with spaces; preserve unrelated content and local conventions.
- Inspect changes and distinguish static validation from an actual application preview or rendered inspection.
- Keep temporary plotting source outside the vault. Persist source when requested or included in an established deliverable convention; a requested note figure includes its necessary exported attachment and embed.
- Record official source evidence separately from technical verification and inferred preparation requirements.

## Requirements

Install dependencies within the user-authorized scope. Check existing tools and environments first; missing tools should be reported with the available fallback.

| Workflow | Requirements and fallback |
| --- | --- |
| Vault workflow CLI | Rust `vault find`, `vault recent`, and `vault check` live under `vault-operator/scripts/vault-cli/`; the sibling `vault-check` binary lives under `obsidian-markdown/scripts/vault-check/`. Git supports `--changed`; `xdg-open` opens notes. See the [Vault CLI reference](vault-operator/references/VAULT-CLI.md). |
| Vault concept search | Rust `vault-search` lives under `vault-operator/scripts/vault-search/`; it requires SQLite with FTS5. See its [usage and installation guide](vault-operator/scripts/vault-search/README.md). |
| Markdown validator | Compiled Rust `vault-check`; Cargo needed only to build/rebuild. Python 3/PyYAML handles uncommon YAML compatibility cases and serves as a full fallback. |
| Bases validator | Python 3 and PyYAML. |
| Canvas validator and CLI detector | Python 3 standard library. |
| Obsidian semantic queries and previews | Obsidian running with its CLI enabled and registered. The detector checks version and help responses; filesystem operations remain available when the CLI is unverified. |
| Note interlinking | Rust `vault find` and `vault-search` (or scoped filesystem search) for candidate discovery; Git for the diff review gate. The Obsidian CLI is optional for resolved-link and backlink checks. |
| Public HTML extraction | Defuddle CLI, or an available web-reading tool when Defuddle is unavailable or the page is unsupported. |
| PDF helper | `uv` and Python 3.11+. PEP 723 metadata declares compatible PyMuPDF and pypdf ranges; these are not locked versions. `uv run` may download dependencies. |
| Additional PDF workflows | Poppler’s `pdfinfo` for inspection; OCRmyPDF, Tesseract, and Ghostscript for OCR. See the PDF workflow reference for other operations. |
| Official syllabus audit | Web access to verify the requested exam, post, and recruitment cycle against official sources. |
| Visual inspection | A suitable renderer or Obsidian preview. Report when rendering could not be verified. |

## Running helpers

Resolve a helper relative to its skill directory, independently of the input file’s location. Keep the working directory unchanged so relative input and output paths continue to refer to your workspace.

For example, replace the placeholder below with the absolute path to the installed skill:

```bash
skills_dir="/absolute/path/to/.agents/skills"
cargo build --release --locked --manifest-path "$skills_dir/vault-operator/scripts/vault-cli/Cargo.toml"
cargo build --release --locked --manifest-path "$skills_dir/obsidian-markdown/scripts/vault-check/Cargo.toml"
cargo build --release --locked --manifest-path "$skills_dir/vault-operator/scripts/vault-search/Cargo.toml"
"$skills_dir/obsidian-markdown/scripts/vault-check/target/release/vault-check" note --quiet "Thermodynamics/Entropy.md"
```

The first build produces `target/release/vault`: ranked title/alias/content search, recent notes, and changed-note syntax/embed checks. Keep `vault` and `vault-check` together at runtime. For terminal access, install both into `~/.local/bin` using the commands in the [Vault CLI reference](vault-operator/references/VAULT-CLI.md). Once installed, Cargo's generated `target/` directories can be removed and recreated by the next build.

Build once, then invoke the installed binaries directly. Batch changed notes in one call; `--quiet` retains warnings and errors while suppressing OK lines. Keep each Rust project with its owning skill; their build outputs are ignored by Git.

The CLI detector can return a machine-readable result:

```bash
skill_dir="/absolute/path/to/obsidian-cli"
python3 "$skill_dir/scripts/check_cli.py" --json
```

Use the returned `executable` path for subsequent Obsidian commands. Do not resolve `obsidian` again through `PATH`, where it may refer to a GUI launcher. An unrecognized response is unverified even if the process exits successfully.

Each skill documents its own helper arguments and validation workflow.

## Validation and tests

Static checks support review; they do not prove visual or semantic correctness:

- Markdown checks frontmatter, fences, math/link delimiter balance, and table structure. Its Rust `preserve BEFORE AFTER` command reports structural losses and frontmatter changes using a pre-edit snapshot; see the Markdown skill for coverage and review semantics.
- Bases checks known structure and formula-reference fields. Duration warnings are heuristic; formulas still need evaluation in Obsidian.
- Canvas checks required fields, types, IDs, edge references, and layout constraints. Unknown extension types produce warnings; malformed values produce errors.
- PDF workflows combine structural inspection with rendered-page checks.

Build the Rust release binaries using the commands above, then run helper regression tests from the collection root with Python 3 and PyYAML available (binary-dependent Python tests are skipped if their binary is absent):

```bash
cargo test --locked --manifest-path obsidian-markdown/scripts/vault-check/Cargo.toml
cargo test --locked --manifest-path vault-operator/scripts/vault-cli/Cargo.toml
cargo test --locked --manifest-path vault-operator/scripts/vault-search/Cargo.toml
python3 -B -m unittest discover -s tests -v
```

The CLI integration tests cover ranking, aliases, URI generation, recency, Git changes/renames, missing embeds, and argument errors in temporary vaults. Tests compare Rust/Python Markdown results on edge cases and 150 seeded generated samples, check quiet mode, read failures, and ordinary YAML without Python. They also cover Markdown code and delimiter handling, Base property/expression distinctions, malformed Canvas values, helper invocation outside the skill directory, and CLI detection. They use temporary fixtures and mocked CLI responses; they do not launch Obsidian or modify vault notes. Passing them does not establish live CLI compatibility or rendered correctness.

## Repository structure

```text
skills/
├── README.md
├── tests/
│   └── test_skill_helpers.py
└── skill-name/
    ├── SKILL.md       # Discovery description and operating instructions
    ├── references/    # Optional documentation loaded when needed
    └── scripts/       # Optional executable helpers
```

Start with the relevant `SKILL.md` for behavior, dependencies, and detailed commands. Individual skills may carry their own licensing terms; review their frontmatter before redistribution.

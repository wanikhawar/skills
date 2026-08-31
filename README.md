# Agent Skills

A collection of reusable agent skills for Obsidian workflows, mechanical-engineering study, web content extraction, and PDF processing.

Each skill lives in its own directory and is defined by a `SKILL.md` file. Some skills also include reference material or helper scripts that should remain alongside the skill.

## Included skills

### General utilities

| Skill | What it does |
| --- | --- |
| [Defuddle](defuddle/SKILL.md) | Extracts clean Markdown from web pages with the Defuddle CLI, removing navigation and other clutter. |
| [PDF](pdf/SKILL.md) | Performs page-aware PDF inspection, extraction, rendering, OCR, structural edits, validation, and Obsidian ingestion. |

### Obsidian

| Skill | What it does |
| --- | --- |
| [JSON Canvas](json-canvas/SKILL.md) | Creates and edits Obsidian `.canvas` files with valid nodes, groups, and connections. |
| [Obsidian Bases](obsidian-bases/SKILL.md) | Creates and edits `.base` files with views, filters, formulas, properties, and summaries. |
| [Obsidian CLI](obsidian-cli/SKILL.md) | Uses the Obsidian CLI to manage vaults, notes, tasks, properties, plugins, and themes. |
| [Obsidian Markdown](obsidian-markdown/SKILL.md) | Authors Obsidian-flavored Markdown, including wikilinks, embeds, callouts, and properties. |
| [Vault Operator](vault-operator/SKILL.md) | Provides a path-first operating protocol for safely working in the author's Obsidian vault. Adapt its personal conventions before using it elsewhere. |

### Mechanical engineering

| Skill | What it does |
| --- | --- |
| [Engineering Visualizer](engineering-visualizer/SKILL.md) | Produces minimal, technically accurate diagrams and plots for mechanical-engineering explanations and notes. |
| [ME Concept Tutor](me-concept-tutor/SKILL.md) | Teaches mechanical-engineering concepts and derivations from first principles, with support for major Indian engineering exams. |
| [Syllabus Gap Audit](syllabus-gap-audit/SKILL.md) | Compares an exam-preparation vault with current official syllabi and helps fill high-value coverage gaps. |

## Installation

Clone the repository:

```bash
git clone https://github.com/wanikhawar/skills.git
```

Then copy either the entire collection or only the skill directories you need into the skills directory used by your agent. Keep each directory intact so its `SKILL.md`, `references/`, and `scripts/` remain together.

For a project-scoped setup that supports the `.agents/skills` convention, for example:

```text
your-project/
└── .agents/
    └── skills/
        ├── obsidian-markdown/
        │   ├── SKILL.md
        │   └── references/
        └── engineering-visualizer/
            └── SKILL.md
```

Skill discovery paths vary between agent clients, so consult your client's documentation if it does not automatically load skills from this location.

## Usage

Once installed, ask your agent for the task in ordinary language. The descriptions in each skill's frontmatter tell a compatible agent when to load it.

Examples:

```text
Extract the useful content from this article as Markdown.

Create an Obsidian Base that lists unfinished thermodynamics notes.

Explain why entropy is a property from first principles.

Audit my Heat Transfer notes against the current GATE and ESE syllabi.

Merge these PDFs and add page numbers.
```

You can also name a skill explicitly when you want the agent to use a particular workflow.

## Requirements and validation

- `defuddle` uses the Defuddle CLI. Check availability before use; dependency installation must be explicitly authorized.
- `obsidian-cli` requires Obsidian 1.12.7+ to be open and the CLI feature to be registered. On Linux, distinguish the registered `~/.local/bin/obsidian` CLI from a distro GUI launcher.
- `syllabus-gap-audit` needs web access to check current official syllabi.
- `vault-operator` contains author-specific vault conventions and should be customized before reuse.
- `pdf/scripts/pdf_ops.py` declares PyMuPDF and pypdf with PEP 723 metadata and runs reproducibly through `uv`; OCR uses OCRmyPDF, Tesseract, and Ghostscript.
- Static validators are bundled for Markdown, Bases, Canvas, and PDF operations. Run them before claiming a file is valid, then preview/render when visual correctness matters.
- Individual skill files may carry their own licensing terms; review frontmatter before redistribution.

## Repository structure

```text
skill-name/
├── SKILL.md       # Trigger description and operating instructions
├── references/    # Optional detailed documentation
└── scripts/       # Optional helper programs
```

Start with the relevant `SKILL.md`; it is the source of truth for that skill's behavior and dependencies.

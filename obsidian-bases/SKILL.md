---
name: obsidian-bases
description: Create, edit, or diagnose Obsidian Bases when the user explicitly refers to Bases, a `.base` file, or a database-like view over note properties. Handle filters, formulas, table/card/list/map layouts, summaries, and embeds; do not activate for ordinary Markdown tables, mathematical formulas, or generic card layouts.
metadata:
  short-description: Reliable Obsidian Bases authoring and repair
---

# Obsidian Bases

Build a Base from the properties the target notes actually contain. A syntactically valid `.base` file can still be useless when property names, types, filters, or formulas do not match the vault.

Current documentation:

- <https://obsidian.md/help/bases>
- <https://obsidian.md/help/bases/syntax>
- <https://obsidian.md/help/bases/functions>
- <https://obsidian.md/help/bases/views>

Read [references/FUNCTIONS_REFERENCE.md](references/FUNCTIONS_REFERENCE.md) only when authoring or debugging formulas.

## Workflow

1. Confirm the user requested a Base or `.base` change. A Base shown as context is not permission to edit it.
2. Inspect the intended dataset: target folder/tags and representative note properties. Reuse exact property names and types; do not invent a parallel schema.
3. Decide whether a standalone `.base` file or an embedded `base` code block best fits the request and local convention.
4. Define the narrowest correct global filters.
5. Add only formulas that improve the requested view.
6. Configure the smallest useful set of views and displayed properties.
7. Run `scripts/validate_base.py`.
8. If the registered Obsidian CLI is available, run `base:views` and `base:query ... format=json`; otherwise open/preview in Obsidian when practical and report that semantic rendering was not verified.

Do not normalize or add frontmatter across source notes unless the user explicitly requests that separate change. If property types are inconsistent, report the conflict or request permission to normalize them.

## Core schema

```yaml
filters:
  and:
    - 'file.ext == "md"'
    - 'file.inFolder("Thermodynamics")'

formulas:
  days_since_review: 'if(last_reviewed, ((today() - date(last_reviewed)) / 86400000).floor(), "")'

properties:
  file.name:
    displayName: Note
  formula.days_since_review:
    displayName: Days since review

summaries:
  roundedAverage: 'values.mean().round(1)'

views:
  - type: table
    name: Review queue
    filters:
      and:
        - 'status != "mastered"'
    order:
      - file.name
      - status
      - last_reviewed
      - formula.days_since_review
    groupBy:
      property: status
      direction: ASC
```

Top-level filters apply to every view; view filters are combined with them using `AND`. A Base with no filters includes every file in the vault.

## Properties and formulas

Use:

- note properties as `status` or `note.status`;
- file properties such as `file.name`, `file.path`, `file.folder`, `file.ext`, `file.ctime`, `file.mtime`, `file.tags`, `file.links`, and `file.backlinks`;
- formula properties as `formula.formula_name`.

Avoid `file.backlinks` when a reverse lookup through `file.links` can express the same query; backlinks are heavier and may not refresh immediately.

Guard optional properties:

```yaml
formulas:
  overdue: 'if(due, date(due) < today() && status != "done", false)'
  days_until_due: 'if(due, ((date(due) - today()) / 86400000).round(0), "")'
```

Date subtraction returns a number of milliseconds. It does **not** return an object with `.days` or `.hours`. Use division by `86400000`, `3600000`, or another appropriate conversion factor.

Use duration values for date addition or subtraction:

```yaml
formulas:
  next_week: 'today() + "7d"'
  two_intervals: 'now() + (duration("12h") * 2)'
```

When duration arithmetic uses a scalar, keep the duration on the left: `duration("5h") * 2`.

## Views

- **Table:** comparisons, sorting, grouping, and summaries.
- **Cards:** gallery-like browsing; choose an existing image property deliberately.
- **List:** compact navigation.
- **Map:** requires Obsidian 1.10+ and the official Maps community plugin. Confirm the plugin is enabled and use a valid coordinates property before creating a map view.
- **Plugin view:** preserve unknown view-specific keys and verify the contributing plugin is installed.

Additional layouts may be contributed by plugins. Do not reject unknown view types or delete unknown configuration while editing an existing Base.

## Embedding

Standalone file:

```markdown
![[Review Queue.base]]
![[Review Queue.base#Due this week]]
```

Inline Base:

````markdown
```base
filters:
  and:
    - 'file.hasTag("review")'
views:
  - type: table
    name: Review
```
````

## Validation contract

Run:

```bash
python3 scripts/validate_base.py path/to/file.base
```

The validator checks YAML structure, filter shape, view names, formula references, and known stale date-duration patterns. It cannot execute the Obsidian formula engine.

After an authorized edit:

- read the file back;
- confirm every displayed `formula.X` is defined;
- confirm source properties exist with compatible types;
- query each important view through the CLI when available;
- preview cards/maps when visual layout matters;
- inspect the diff for unrelated changes.

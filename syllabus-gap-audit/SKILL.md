---
name: syllabus-gap-audit
description: Audit an Obsidian mechanical-engineering exam-prep vault, folder, MOC, or note cluster against current official syllabi for GATE, ESE, RRB JE, SSC JE, JKSSB, or ISRO. Produce a traceable coverage report by default, and create or patch notes only when the user explicitly asks to fill identified gaps.
license: MIT
metadata:
  short-description: Official-syllabus audit for ME exam notes
---

# Syllabus Gap Audit

Compare the requested vault scope against the current official syllabus for the exact exams, posts, and recruitment cycles in scope. Search hits are candidates for inspection, not proof of coverage.

For vault work, follow applicable `AGENTS.md` instructions and `vault-operator` when installed. Standalone fallback: edit only within the user’s requested scope, preserve unrelated content and conventions, then read back changes and inspect the diff.

## Modes and authorization

- **Audit:** collect sources, normalize requirements, inspect vault coverage, and report gaps. This is the default.
- **Audit and fill:** perform the audit, then make the authorized note changes.
- **Fill from an existing audit:** verify the audit is still applicable, then implement only the requested priorities.

An audit request alone does not authorize note creation or modification. An active note, folder, or MOC is context only.

Load these companion skills when installed and relevant:

- `vault-operator` for path resolution, consent, safe edits, and validation;
- `obsidian-markdown` for authorized note changes;
- `pdf` when an official syllabus or notification is a PDF;
- `me-concept-tutor` when filling a gap requires a derivation or technically deep explanation;
- `engineering-visualizer` only when a visual materially improves a missing concept.

If a companion skill is unavailable, use available tools with the standalone policy above, exact path resolution, source/page evidence, and the technical checks below. Report unavailable validation rather than claiming it succeeded. Do not expand into lectures, research papers, OCR, or dashboards unless the task requires them.

## Establish exact scope

Resolve:

1. vault root by locating `.obsidian/`, never from a hard-coded platform path;
2. target notes/folders/MOC;
3. exam set;
4. current or requested notification cycle;
5. exact post/discipline and recruitment route where relevant;
6. whether the user wants report-only, high-priority filling, or complete filling.

JKSSB and ISRO coverage is often post- or route-specific. Do not silently substitute a generic Mechanical Engineering syllabus for a specific advertisement.

## Source protocol

Current syllabi, notifications, and recruitment routes are time-sensitive, so verify them on the internet. Record the research date.

Prefer, in order:

1. official exam-body syllabus page or PDF for the requested cycle;
2. official notification/admission brochure containing the syllabus;
3. the most recent prior official cycle when the new one is not published, clearly labeled;
4. a reputable secondary source only to locate or cross-check an official source.

Official bodies include the current GATE organizing institute, UPSC, SSC, the relevant Railway Recruitment Board/CEN, JKSSB, and ISRO/ICRB. Discover the current official domain dynamically rather than hard-coding an organizing institute or year.

For every source capture:

- exam and paper/post;
- year, cycle, advertisement, or CEN number;
- official status and URL;
- PDF page and section when applicable;
- recruitment route and scope caveats;
- retrieval date.

Do not use search snippets, coaching answer pages, or copied answer keys as syllabus evidence.

## Normalize requirements

Create an exam-useful matrix at the smallest granularity supported by the official wording:

```text
Area | Official topic/subtopic | Exam/post | Cycle | Source page/section | Officially stated depth | Inferred preparation depth + evidence | Vault evidence | Coverage | Correctness | Navigation | Priority | Action
```

Common Mechanical Engineering areas include mathematics, engineering mechanics, strength of materials, theory of machines/vibrations, design, fluids/hydraulic machines, thermodynamics, heat transfer, RAC, power/engines/turbomachinery, materials/metallurgy, manufacturing, metrology, industrial engineering/OR, drawing/CAD, and exam-specific aptitude or general awareness.

Do not force every exam into the same granularity. Preserve differences in wording and breadth. Record officially stated depth only when the source specifies it; otherwise write “not specified.” Keep inferred preparation depth separate and cite its basis, such as an inspected official sample question or identified PYQ. Label judgment as inference; leave depth unresolved when evidence is insufficient.

## Audit the vault

1. Resolve exact paths using `vault-operator`.
2. Read the target MOC/overview first when one exists.
3. List notes in scope and search filenames using syllabus terms and synonyms.
4. Search contents to identify candidate coverage.
5. Read each candidate section before assigning status.
6. Record the exact note and heading that provide evidence.
7. Check discoverability through MOC links when navigation is in scope.

Assess each dimension independently:

| Dimension | Values and evidence standard |
| --- | --- |
| Coverage | **Complete:** all in-scope requirements have usable content. **Partial:** some required content is absent. **Missing:** no useful content after synonym and content inspection. **Not assessed:** inspection is incomplete or scope is unresolved. |
| Correctness | **Verified:** the relevant content passed the technical checks below. **Uncertain:** evidence or verification is insufficient. **Incorrect:** a specific error or missing critical validity condition is demonstrated. |
| Navigation | **Anchored:** coverage has a clear authoritative note/section and, when in scope, a working MOC path. **Scattered:** relevant material is distributed without a clear anchor. Use **Not assessed** when navigation is out of scope or no content exists. |

A topic can be complete, verified, and scattered. Complete coverage is not proof of correctness. Use correctness “Uncertain” for missing or uninspected content, explaining that there was nothing to verify; never label unreviewed material incorrect. Record exact note/heading evidence and reasons for each judgment. A matching filename, tag, or keyword cannot establish complete coverage.

## Prioritize transparently

Priority must follow the user's target exams and documented evidence. Explain each priority rather than applying a universal score.

- **High:** a whole required area is absent, a demonstrated incorrect concept affects many problems, or a topic is central to several target exams.
- **Medium:** a required subtopic or qualification is missing from otherwise useful notes.
- **Low:** peripheral, post-specific, or low-depth material for the current target, unless the user prioritizes that exam/post.

Use PYQ frequency only when supported by actual PYQ evidence; do not infer frequency from coaching claims.

## Filling authorized gaps

Choose the least disruptive home:

1. Patch an authoritative note for a missing subtopic.
2. Create an atomic note for a genuinely distinct major topic.
3. Add a MOC link only when navigation is part of the requested fix.
4. Use a compact addendum when the existing structure should remain intact.
5. Link to one authoritative derivation instead of duplicating it across notes.

Inspect nearby notes before choosing headings, frontmatter, tags, callouts, or filename style. Do not apply a universal template or tag every note with every exam. Add only properties that match the local schema and the exams actually relevant to that note.

For source incorporation, use the shared completeness contract. Standalone fallback: retain every distinct correct in-scope point, including meaningful figures/tables, and consolidate duplicates without losing qualifications.

For a large authorized audit, work in reviewable phases: sources and matrix, vault coverage, high-priority corrections, medium-priority coverage, then navigation cleanup. Explicit permission for a complete bulk edit is sufficient; otherwise pause before a mutation set whose scope cannot be safely reviewed.

## Technical verification

Before saving substantive engineering content:

- derive or verify equations from accepted principles;
- check dimensions, signs, limiting cases, reference states, and assumptions;
- verify empirical correlations and applicability ranges from a reliable source;
- distinguish official syllabus evidence from textbook support for technical content;
- mark unresolved empirical claims as unverified instead of guessing.

Follow `obsidian-markdown` for math and note syntax when installed. Standalone fallback: use `$...$` and `$$...$$`; convert MCQs into declarative insights with conditions and provenance.

## Report

Include:

```markdown
## Sources checked

| Exam/post | Cycle | Official source | Page/section | Scope note |
| --- | --- | --- | --- | --- |

## Coverage

| Official topic | Vault evidence | Coverage | Correctness | Navigation | Priority | Recommended action |
| --- | --- | --- | --- | --- | --- | --- |

## Changes made

- Created: `exact/path.md`
- Patched: `exact/path.md#Heading`

## Remaining uncertainty

- ...
```

For incorporation edits, use the shared reporting contract. Standalone fallback: report additions, consolidation, and corrections/omissions with reasons, including when none were omitted.

After edits, apply the shared validation contract. Run the Markdown validator when installed and check new links; distinguish static validation from an actual Obsidian preview.

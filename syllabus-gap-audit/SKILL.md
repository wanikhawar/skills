---
name: syllabus-gap-audit
description: Use when auditing an Obsidian mechanical-engineering exam-prep vault or folder against current internet-sourced syllabi for GATE, ESE, RRB JE, SSC JE, JKSSB, and ISRO, then creating or patching notes to fill missing high-yield gaps.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [obsidian, syllabus, gate, ese, rrb, ssc, jkssb, isro, mechanical-engineering, exam-prep]
    related_skills: [obsidian, ocr-and-documents, youtube-content, arxiv]
---

# Syllabus Gap Audit for ME Exam-Prep Vaults

## Overview

Use this skill to audit a Mechanical Engineering Obsidian vault, folder, MOC, or note cluster against current syllabi for:

- GATE Mechanical Engineering
- ESE / IES Mechanical Engineering
- RRB JE Mechanical / allied technical syllabus
- SSC JE Mechanical Engineering
- JKSSB Mechanical / JE / technical posts where applicable
- ISRO Scientist/Engineer / Technical Assistant Mechanical where applicable

The workflow is internet-grounded: do not rely only on memory of the syllabus. Search the web, prefer official PDFs/pages, normalize the syllabus into a topic matrix, compare it against the vault, then fill gaps as clean Obsidian study notes or focused addenda.

This is designed for an exam-prep vault where notes should be concise but technically correct, with dollar-delimited LaTeX math, wikilinks, exam traps, formulas, and high-yield tables.

## When to Use

Use this skill when the user asks things like:

- "Audit Heat Transfer against all syllabi and fill missing topics."
- "Check this folder for GATE/ESE/RRB/SSC/JKSSB/ISRO gaps."
- "Create missing notes from the official syllabus."
- "Compare my Production notes with GATE, ESE, SSC JE, RRB JE and ISRO."
- "Tell me what syllabus topics are absent and patch the notes."

Do not use this skill for:

- Pure factual checking of formulas without a syllabus comparison.
- A single-note style rewrite with no exam syllabus requirement.
- Research-paper depth unless the user asks for PhD-level extension after exam coverage is complete.

## Required Companion Skills

Load `obsidian` before working in the vault.

Also load these when relevant:

- `ocr-and-documents` if using PDFs, scanned syllabi, handbooks, or PYQ PDFs.
- `youtube-content` if the user asks to supplement gaps from lectures.
- `arxiv` only for advanced conceptual depth, not for routine exam syllabus coverage.

## Source Hierarchy

Always prefer official and current sources. Use third-party coaching sites only as discovery aids or cross-checks, never as sole authority.

Priority order:

1. Official exam body PDF/page.
2. Official notification/admission brochure containing syllabus.
3. Official previous-year notification when the current one is not available.
4. Reputable coaching page only for cross-checking or finding official document names.
5. Never use unsourced blog snippets as final authority.

Recommended official source targets:

| Exam | Preferred official source |
|---|---|
| GATE ME | Current GATE official website by organizing IIT/IISc; official syllabus PDF/page for Mechanical Engineering, paper code ME |
| ESE / IES ME | UPSC Engineering Services Examination notification and syllabus PDF |
| RRB JE | Latest Railway Recruitment Board CEN notification/syllabus PDF for JE/technical categories |
| SSC JE ME | Staff Selection Commission JE notification/syllabus PDF |
| JKSSB | JKSSB official notification/syllabus for the specific post; if generic, state the ambiguity clearly |
| ISRO ME | ISRO/ICRB official recruitment notification, Scientist/Engineer Mechanical syllabus if provided; otherwise identify whether selection is based on discipline fundamentals, written test, interview, or GATE route |

## Internet Search Protocol

1. Record the research date in the audit note or final report.
2. Search official sources first with targeted queries:

```text
site:gate2026.iitg.ac.in Mechanical Engineering syllabus PDF GATE ME
site:gate2025.iitr.ac.in Mechanical Engineering syllabus PDF GATE ME
site:upsc.gov.in Engineering Services Examination Mechanical Engineering syllabus PDF
site:ssc.gov.in Junior Engineer Mechanical syllabus PDF
site:rrb*.gov.in CEN JE Mechanical syllabus PDF
site:jkssb.nic.in mechanical syllabus junior engineer PDF
site:isro.gov.in ICRB Scientist Engineer Mechanical syllabus PDF
site:isro.gov.in Technical Assistant Mechanical syllabus PDF
```

3. If exact current-year official pages are not easily found, broaden the query but keep official source preference:

```text
GATE Mechanical Engineering syllabus official PDF
UPSC ESE Mechanical Engineering syllabus official PDF
SSC JE Mechanical Engineering syllabus official PDF
RRB JE Mechanical Engineering syllabus official PDF
JKSSB Mechanical Engineering syllabus official PDF
ISRO Scientist Engineer Mechanical syllabus official PDF
```

4. For each source, capture:

- Exam name
- Year / notification cycle
- URL
- Whether it is official or third-party
- Mechanical paper/post applicability
- Any ambiguity about recruitment route or syllabus scope

5. If a syllabus is route-dependent, do not invent missing details. State the route:

- GATE-based recruitment: use GATE ME syllabus plus General Aptitude where applicable.
- Written test with discipline syllabus: use the official discipline topics.
- Post-specific JKSSB/ISRO notification: use only the syllabus for that post or clearly flag that the user must provide the exact post notification.

## Normalize the Syllabus

Create a normalized topic matrix before editing notes. Use exam-useful granularity.

Suggested ME topic buckets:

- Engineering Mathematics
- Engineering Mechanics
- Strength of Materials
- Theory of Machines and Vibrations
- Machine Design
- Fluid Mechanics and Hydraulic Machines
- Thermodynamics
- Heat Transfer
- Refrigeration and Air Conditioning
- Power Plant / IC Engines / Compressors / Turbines
- Materials Science / Metallurgy
- Manufacturing / Production Engineering
- Metrology and Inspection
- Industrial Engineering / Operations Research
- Engineering Drawing / CAD where relevant
- General Aptitude / Reasoning / General Awareness where relevant
- Post-specific technical topics for JKSSB/ISRO/SSC/RRB

For each topic/subtopic, store:

```text
Topic | Subtopic | GATE | ESE | RRB | SSC JE | JKSSB | ISRO | Source URLs | Priority
```

Priority rules:

- HIGH: appears in multiple exams, or appears in GATE/ESE/ISRO core, or is a whole missing major topic.
- MEDIUM: appears in one or two exams or is a missing subtopic within an existing note.
- LOW: rare, post-specific, or peripheral unless the user is targeting that exam specifically.

## Vault Audit Workflow

1. Resolve the vault path using the `obsidian` skill rules. In Khawar's vault, the known path is:

```text
/Users/wanikhawar/Library/CloudStorage/GoogleDrive-wanikhawar@gmail.com/My Drive/Academic
```

2. Resolve the target scope:

- Whole vault
- Subject folder
- MOC note
- Specific note cluster

3. If folder names are ambiguous, list candidate folders and clarify. Important example: `Materials/` may mean `Strength of Materials/` or `Material Science/`.

4. List all `.md` files in the target scope with `search_files(target="files")`.

5. Read the MOC/overview note first if present.

6. Read all relevant notes, in batches when large. For truncated notes, read remaining pages with offsets.

7. Search both filenames and contents for each normalized syllabus topic using synonyms and variants. Examples:

- `Rankine|vapour power|reheat|regeneration`
- `LMTD|NTU|effectiveness|heat exchanger`
- `Euler|Rankine column|slenderness`
- `inventory|EOQ|ABC|VED|queuing|PERT|CPM`
- `metrology|fits|tolerance|gauge|surface roughness`

8. Build coverage status:

| Status | Meaning |
|---|---|
| Covered | Good enough for exam revision |
| Partial | Present but missing formula, assumptions, classification, diagram logic, or exam traps |
| Scattered | Present across notes but no clean anchor note/MOC link |
| Missing | No useful note coverage found |
| Wrong/Risky | Formula or concept appears incorrect or misleading |

## Gap-Filling Rules

When the user explicitly asks to fill gaps, edit the vault after the audit. Do not stop at reporting unless the scope is very large or ambiguous.

Choose the least disruptive edit:

1. Patch an existing authoritative note when the gap is a subtopic.
2. Create a dedicated atomic note when the gap is a major topic.
3. Add a MOC link when the note exists but is not discoverable.
4. Create a compact `High-Yield Addendum` section when PYQ/syllabus importance is high but the note structure should be preserved.
5. Avoid duplicating full derivations across multiple notes. Prefer a short callout and wikilink to the authoritative note.

For very broad audits, proceed in phases:

- Phase 1: internet source collection + syllabus matrix
- Phase 2: vault coverage matrix
- Phase 3: fill HIGH gaps
- Phase 4: fill MEDIUM gaps
- Phase 5: cleanup MOC links and broken links

If filling everything would create more than about 8-10 substantial notes or edits, ask the user to choose a phase unless they already gave explicit permission for a full bulk edit.

## Note Format for Created or Patched Notes

Use this structure unless the folder has a stronger existing convention:

```markdown
---
subject: <Subject Name>
tags: [GATE, ESE, RRB, SSC-JE, JKSSB, ISRO]
exam_priority: high
created: YYYY-MM-DD
---

# Note Title

## Core Idea

Concise definition and why it matters.

## Syllabus Relevance

| Exam | Relevance |
|---|---|
| GATE | ... |
| ESE | ... |
| RRB JE | ... |
| SSC JE | ... |
| JKSSB | ... |
| ISRO | ... |

## Key Formulas

Use dollar-delimited LaTeX only.

Inline: `$q = hA(T_s - T_\infty)$`

Display:

$$
q = kA\frac{\Delta T}{L}
$$

## Assumptions and Limitations

- ...

## Classifications / Comparison Table

| Case | Formula / Feature | Use |
|---|---|---|
| ... | ... | ... |

## Exam Traps

> [!warning] Exam Trap
> Common wrong assumption or frequently confused formula.

## Related Notes

- [[Relevant Note 1]]
- [[Relevant Note 2]]
```

Respect existing folder conventions. If existing notes do not use frontmatter, either follow the local convention or add minimal frontmatter only when it will not make the vault inconsistent.

## Exam-Specific Emphasis

### GATE

Focus on conceptual clarity, standard derivations, dimensionless numbers, numerical formula use, and General Aptitude when doing whole-exam dashboards. Mechanical paper code ME is the baseline.

### ESE / IES

Include broader coverage and more conventional theory than GATE. Emphasize definitions, classifications, descriptive questions, assumptions, and engineering applications.

### RRB JE

Keep notes more direct and fact-oriented. Add one-line definitions, classifications, applications, instruments, materials/process facts, and practical engineering points.

### SSC JE

Balance numerical formulas with direct technical facts. Include measurement, workshop/process, machine elements, thermal/fluid basics, and conventional JE-level facts.

### JKSSB

Treat as post-specific. Do not assume a single universal mechanical syllabus. If the exact post is unknown, flag ambiguity and use common JE Mechanical coverage as provisional.

### ISRO

Treat recruitment route carefully. If the official notification uses a written test, cover core engineering fundamentals at high precision. If GATE-based, use GATE ME as the technical baseline. If no detailed syllabus is provided, state that explicitly and avoid inventing a pseudo-official syllabus.

## Formula and Fact Verification

Before writing a formula-heavy gap note:

1. Cross-check with at least one reliable source: official syllabus for topic existence, standard ME knowledge for formulas, or a trusted textbook/handbook if available in the vault.
2. Use correct assumptions and validity ranges where relevant.
3. For correlations, include conditions: laminar/turbulent, geometry, Reynolds/Prandtl range if known.
4. For design formulas, specify units and sign conventions when exam-relevant.
5. Avoid overfitting to one coaching source.

## Reporting Format

Use this report shape before or after edits:

```markdown
## Sources Checked

| Exam | Source | Official? | Year/Cycle | Notes |
|---|---|---:|---|---|
| GATE ME | URL | Yes | 2026 | ... |

## Coverage Summary

| Syllabus Area | Vault Coverage | Status | Priority | Action |
|---|---|---|---|---|
| Heat Exchangers | Existing note covers LMTD, misses NTU | Partial | HIGH | Patched note |

## High-Priority Gaps Filled

- Created: `Subject/Topic.md`
- Patched: `Subject/Existing Note.md`

## Remaining Gaps

| Priority | Gap | Why it matters | Suggested next action |
|---|---|---|---|
| MEDIUM | ... | ... | ... |
```

When editing files, always state exact paths changed.

## Verification Checklist

After filling gaps:

- [ ] Re-read every modified/created note.
- [ ] Check that LaTeX uses `$...$` and `$$...$$`, not `\[...\]`.
- [ ] Check markdown tables render: header row + separator row + equal column counts.
- [ ] Check wikilinks point to existing notes where practical.
- [ ] Check no duplicate note was created because of filename/path mismatch.
- [ ] Check no source was represented as official unless it really was official.
- [ ] If using PDFs/OCR, confirm extracted text matches the intended pages/section.
- [ ] Run `git diff --stat` and inspect `git diff` for modified notes when practical.

## Common Pitfalls

1. **Inventing syllabi from memory.** Always search current official sources first.

2. **Mixing recruitment routes.** ISRO, PSU, and some state posts may be GATE-based, written-test-based, or post-specific. State the route clearly.

3. **Using coaching pages as official sources.** Third-party pages can help discovery but must not be final authority.

4. **Treating JKSSB as one fixed syllabus.** JKSSB syllabi are often post/advertisement-specific. Ask for or search the exact post when needed.

5. **Creating redundant notes.** Search filenames and content before creating. If content exists but is scattered, consolidate with links instead.

6. **Overwriting the user's note style.** Preserve existing MOCs, frontmatter, tags, headings, wikilinks, and useful explanations.

7. **Breaking LaTeX with automated patches.** For brace-heavy LaTeX edits, use Python raw file I/O as described in the `obsidian` skill rather than fragile patch replacements.

8. **Filling low-yield gaps first.** Prioritize topics appearing across GATE/ESE/SSC/RRB/ISRO or missing major folders before niche post-specific content.

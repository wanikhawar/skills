# PDF Workflows

Read only the section needed for the current task.

## OCR and searchable PDFs

Preflight:

```bash
ocrmypdf --version
tesseract --version
gs --version
```

Create a new searchable output:

```bash
ocrmypdf --skip-text --rotate-pages --deskew "scan.pdf" "scan-searchable.pdf"
```

Use `--skip-text` for mixed PDFs so existing text pages are preserved. Use `--force-ocr` only when replacing an unreliable text layer is intentional. Select the correct language with `-l`; install the corresponding Tesseract language data rather than silently using English.

Validate with `pdfinfo`, text extraction, and rendered page comparison. OCR success means a text layer was produced, not that equations or tables were transcribed correctly.

## Tables

Use PyMuPDF's table finder or a narrowly configured extraction method. Always inspect the rendered page because ruling lines, merged cells, multi-row headers, repeated headers, footnotes, and units are frequently lost.

For each table preserve:

- title/caption and PDF page;
- complete headers and units;
- row/column relationships;
- footnotes, symbols, and qualifiers;
- blank versus zero values;
- uncertainty where cells are visually ambiguous.

Do not convert a table into prose when the comparisons depend on row/column structure.

## Figures and equations

Render the source page at adequate resolution. Extract or describe axes, units, curve labels, arrows, regions, state points, dimensions, and caption constraints. Do not infer precise values from a qualitative curve.

For equations, compare extracted text against the rendered formula. OCR commonly confuses minus signs, decimal points, primes, Greek letters, subscripts, and superscripts.

## Creating PDFs

Choose an authoring path that matches the document:

- Existing office document or polished report: create/edit the source document and export through LibreOffice.
- HTML/CSS layout: use WeasyPrint when installed and validate font embedding and pagination.
- Programmatic page drawing or overlays: use PyMuPDF.
- Existing PDF page operations: use pypdf through `scripts/pdf_ops.py`.

Avoid rasterizing text unless the requested output is intentionally image-only. Embed fonts when portability matters. For mechanical-engineering notation, inspect every page containing Greek letters, subscripts, superscripts, or uncommon symbols.

## Forms

First determine whether AcroForm fields exist:

```python
from pypdf import PdfReader

reader = PdfReader("form.pdf")
fields = reader.get_fields() or {}
print(fields.keys())
```

Map field names and allowed values before writing. Radio buttons, checkboxes, and choice fields often require export values different from their visible labels. Write a new output, regenerate appearances if required, and render every filled page.

For a non-fillable form, prefer a transparent annotation/overlay workflow and validate coordinate transforms against rendered pages. Never guess field coordinates from extracted text alone.

## Encryption and passwords

Confirm that the user is authorized to decrypt or modify the file. Use a tool mode that prompts for the password or reads it from a protected file descriptor/file. Do not place the password directly after `--password=` or inside source code.

After encryption/decryption, verify whether opening, printing, copying, annotating, and form filling behave as requested. Do not represent PDF permission flags as strong DRM.

## Obsidian source notes

Use page-aware links:

```markdown
Source: [[Sources/Official Syllabus.pdf#page=8]]

![[Sources/Official Syllabus.pdf#page=8&height=500]]
```

When printed and PDF page numbers differ, write both: “printed p. 6; PDF p. 8.” Preserve the original source URL and retrieval date in the note's existing property/citation convention when requested.

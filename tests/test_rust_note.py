"""Rust/Python behavior comparisons; build vault-check before running."""
import importlib.util
import os
from pathlib import Path
import random
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'obsidian-markdown/scripts/vault-check/target/release/vault-check'
SPEC = importlib.util.spec_from_file_location('legacy_note', ROOT / 'obsidian-markdown/scripts/validate_note.py')
LEGACY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LEGACY)

@unittest.skipUnless(BINARY.is_file(), 'build the Rust release binary first')
class RustNoteTests(unittest.TestCase):
    def run_note(self, path, *options, **kwargs):
        return subprocess.run([str(BINARY), 'note', *options, str(path)], capture_output=True, text=True, **kwargs)

    def test_behavior_parity(self):
        cases = [
            'Use `$$`, `[[`, and `\\[` literally.',
            'Use ``a ` $$ [[ \\[ b`` literally.', 'Use `a\n$$` literally.',
            '` example $$', '`$$` then $$x',
            '```text\nexample\n```not-a-close\n', '```text\n$$\n````  \n',
            '| A | B |\n| --- | --- |\n| [[a\\|b]] | 2 |',
            '| A | B |\n| --- | --- |\n| 1 |',
            '中文 `中文 $$` $x$', '$$$', '\\$$', '[[x',
            '---\nx: [\n---', '---\n[]\n---', '---\n---',
            '---\nx: true\ny: 2026-09-08\n---',
            '---\nx: !unknown a\n---', '---\nx: &a [*a]\n---',
            '---\nx: {a: 1, a: 2}\n---', '---\n? [a, b]\n: c\n---',
            '---\nx: |\n  !not-a-tag\n---', '---\nx: "!not-a-tag"\n---',
            '---\nx: &defaults {a: 1}\ny: {<<: *defaults}\n---',
        ]
        rng = random.Random(7)
        tokens = ['text', '`', '``', '$', '$$', '\\$', '[[', ']]', '\n', '中文', '|', '\\|']
        cases += [' '.join(rng.choices(tokens, k=30)) for _ in range(150)]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'note with spaces.md'
            for text in cases:
                with self.subTest(text=text):
                    path.write_text(text)
                    errors, warnings = LEGACY.validate(path)
                    result = self.run_note(path, cwd=directory)
                    self.assertEqual(result.returncode, int(bool(errors)), result.stdout + result.stderr)
                    # YAML parsers have different diagnostic wording; match category.
                    if text.startswith('---'):
                        self.assertEqual('WARNING' in result.stdout, bool(warnings))
                    else:
                        expected = [f'WARNING {path}: {w}' for w in warnings]
                        expected += [f'ERROR {path}: {e}' for e in errors]
                        if not errors: expected += [f'OK {path}']
                        self.assertEqual(result.stdout.splitlines(), expected)

    def test_cli_and_read_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / '-note.md'
            path.write_text('A valid note.')
            self.assertEqual(self.run_note('-note.md', '--quiet', '--', cwd=directory).stdout, '')
            path.write_text('$ambiguous')
            self.assertIn('WARNING', self.run_note(path, '--quiet').stdout)
            path.write_bytes(b'\xff')
            self.assertEqual(self.run_note(path).returncode, 1)
            path.unlink()
            self.assertEqual(self.run_note(path).returncode, 1)
            self.assertEqual(subprocess.run([str(BINARY),'note'], capture_output=True).returncode, 2)

    def test_ordinary_yaml_needs_no_python(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'note.md'
            path.write_text('---\ntags: [thermo]\nreviewed: 2026-09-08\n---\n$x$')
            self.assertEqual(self.run_note(path, env={**os.environ,'PATH':directory}).returncode, 0)

    def test_malformed_tag_is_error_not_crash(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'note.md'
            path.write_text('---\nx: !!int bad\n---')
            result = self.run_note(path)
            self.assertEqual(result.returncode, 1)
            self.assertIn('invalid frontmatter YAML', result.stdout)

@unittest.skipUnless(BINARY.is_file(), 'build the Rust release binary first')
class RustPreserveTests(unittest.TestCase):
    def test_cli_review_and_read_only_behavior(self):
        with tempfile.TemporaryDirectory() as directory:
            before = Path(directory) / 'before note.md'
            after = Path(directory) / 'after note.md'
            before.write_text('# Topic\n![[figure.svg]]\nESE 2020\n')
            after.write_text('# Topic\nNew prose.\n')
            originals = before.read_bytes(), after.read_bytes()
            result = subprocess.run([str(BINARY), 'preserve', '--quiet', str(before), str(after)],
                                    cwd=directory, capture_output=True, text=True,
                                    env={**os.environ, 'PATH': directory})
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn('embed', result.stdout)
            self.assertIn('exam reference', result.stdout)
            self.assertEqual(originals, (before.read_bytes(), after.read_bytes()))

    def test_clean_quiet_and_literal_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, '-before.md').write_text('# Topic\n')
            Path(directory, '-after.md').write_text('# Topic\nNew prose.\n')
            result = subprocess.run([str(BINARY), 'preserve', '--quiet', '--', '-before.md', '-after.md'],
                                    cwd=directory, capture_output=True, text=True)
            self.assertEqual((result.returncode, result.stdout, result.stderr), (0, '', ''))

    def test_unavailable_or_invalid_input(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'note.md'
            path.write_bytes(b'\xff')
            for args in [[], [str(path)], [str(path), str(path)],
                         [str(path), str(path), str(path)], ['--unknown']]:
                result = subprocess.run([str(BINARY), 'preserve', *args], cwd=directory,
                                        capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
            path.unlink()
            result = subprocess.run([str(BINARY), 'preserve', str(path), str(path)], capture_output=True)
            self.assertEqual(result.returncode, 2)

if __name__ == '__main__':
    unittest.main()

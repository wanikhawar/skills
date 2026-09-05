"""Behavioral regressions; temporary files only, no Obsidian process launches."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(relative):
    spec = importlib.util.spec_from_file_location(Path(relative).stem, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


NOTE = load('obsidian-markdown/scripts/validate_note.py')
BASE = load('obsidian-bases/scripts/validate_base.py')
CANVAS = load('json-canvas/scripts/validate_canvas.py')
CLI = load('obsidian-cli/scripts/check_cli.py')


class ValidationTests(unittest.TestCase):
    def validate(self, module, content, suffix):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / ('sample' + suffix)
            path.write_text(content)
            return module.validate(path)

    def test_inline_code_is_literal(self):
        for text in ['Use `$$`, `[[`, and `\\[` literally.',
                     'Use ``a ` $$ [[ \\[ b`` literally.',
                     'Use `a\n$$` literally.']:
            with self.subTest(text=text):
                self.assertEqual(self.validate(NOTE, text, '.md'), ([], []))

    def test_unmatched_inline_code_does_not_hide_math(self):
        errors, _ = self.validate(NOTE, '` example $$', '.md')
        self.assertTrue(errors)

    def test_real_math_error_remains_visible(self):
        errors, _ = self.validate(NOTE, '`$$` then $$x', '.md')
        self.assertTrue(errors)

    def test_fence_with_trailing_text_is_not_closing(self):
        errors, _ = self.validate(NOTE, '```text\nexample\n```not-a-close\n', '.md')
        self.assertTrue(any('unclosed' in error for error in errors))

    def test_longer_fence_closes(self):
        self.assertEqual(self.validate(NOTE, '```text\n$$\n````  \n', '.md'), ([], []))

    def test_base_property_names_and_labels_are_not_expressions(self):
        text = '''properties:
  note.hours:
    displayName: formula.missing
views:
  - type: table
    name: formula.missing.days
    order: [file.name, note.hours]
'''
        self.assertEqual(self.validate(BASE, text, '.base'), ([], []))

    def test_formula_string_literals_are_not_references(self):
        text = '''formulas:
  label: '\"formula.missing (today() - date(x)).days\"'
views:
  - type: table
    name: Test
    order: [formula.label]
'''
        self.assertEqual(self.validate(BASE, text, '.base'), ([], []))

    def test_undefined_expression_reference_is_reported(self):
        text = 'filters: formula.missing > 0\nviews:\n  - {type: table, name: Test}\n'
        errors, _ = self.validate(BASE, text, '.base')
        self.assertTrue(any('undefined' in error for error in errors))

    def test_undefined_display_reference_is_reported(self):
        text = 'views:\n  - {type: table, name: Test, order: [formula.missing]}\n'
        errors, _ = self.validate(BASE, text, '.base')
        self.assertTrue(any('undefined' in error for error in errors))

    def test_duration_heuristic_warns(self):
        text = '''formulas:
  elapsed: '(today() - date(start)).days'
views:
  - {type: table, name: Test, order: [formula.elapsed]}
'''
        errors, warnings = self.validate(BASE, text, '.base')
        self.assertFalse(errors)
        self.assertTrue(any('duration' in warning for warning in warnings))

    def test_canvas_malformed_types_report_errors_without_crashing(self):
        for field, value in [('color', []), ('color', {}), ('type', []),
                             ('type', None), ('x', True)]:
            node = dict(id='0123456789abcdef', type='text', text='Hello',
                        x=0, y=0, width=100, height=100)
            node[field] = value
            with self.subTest(field=field, value=value):
                errors, _ = self.validate(CANVAS, json.dumps({'nodes': [node], 'edges': []}), '.canvas')
                self.assertTrue(errors)

    def test_canvas_bad_edge_types(self):
        for field in ['fromNode', 'fromSide', 'fromEnd', 'color']:
            edge = dict(id='0123456789abcdef', fromNode='missing', toNode='missing')
            edge[field] = []
            errors, _ = self.validate(CANVAS, json.dumps({'nodes': [], 'edges': [edge]}), '.canvas')
            self.assertTrue(errors)

    def test_canvas_unknown_extension_is_warning(self):
        node = dict(id='0123456789abcdef', type='plugin-card', x=0, y=0, width=100, height=100)
        errors, warnings = self.validate(CANVAS, json.dumps({'nodes': [node]}), '.canvas')
        self.assertFalse(errors)
        self.assertTrue(warnings)

    def test_helper_runs_outside_skill_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, 'note.md').write_text('A valid note.\n')
            result = subprocess.run(['python3', str(ROOT / 'obsidian-markdown/scripts/validate_note.py'), 'note.md'],
                                    cwd=directory, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)


class CliTests(unittest.TestCase):
    def probe(self, responses):
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / 'registered obsidian'
            executable.write_text('#!/bin/sh\n')
            executable.chmod(0o700)
            output = io.StringIO()
            with patch.object(CLI, 'candidates', return_value=iter([executable])), \
                 patch.object(CLI.subprocess, 'run', side_effect=responses) as run, \
                 patch('sys.argv', ['check_cli.py', '--json']), \
                 contextlib.redirect_stdout(output), contextlib.redirect_stderr(io.StringIO()):
                status = CLI.main()
            return status, output.getvalue(), run.call_args_list, str(executable)

    def test_successful_noise_is_not_a_cli(self):
        status, output, calls, _ = self.probe([subprocess.CompletedProcess([], 0, 'GUI launched', '')])
        self.assertEqual(status, 1)
        self.assertEqual(output, '')
        self.assertEqual(len(calls), 1)

    def test_version_without_cli_help_is_rejected(self):
        status, _, _, _ = self.probe([subprocess.CompletedProcess([], 0, '1.12.7', ''),
                                    subprocess.CompletedProcess([], 0, 'Generic help', '')])
        self.assertEqual(status, 1)

    def test_verified_path_is_returned(self):
        status, output, calls, executable = self.probe([
            subprocess.CompletedProcess([], 0, '1.12.7 (installer 1.12.7)', ''),
            subprocess.CompletedProcess([], 0, 'Obsidian CLI\n  read path=<path>\n  search query=<text>\n  vault\n', '')])
        self.assertEqual(status, 0)
        self.assertEqual(json.loads(output)['executable'], executable)
        self.assertTrue(all(call.args[0][0] == executable for call in calls))

    def test_timeout_fails_without_retrying(self):
        status, _, calls, _ = self.probe([subprocess.TimeoutExpired('version', 5)])
        self.assertEqual(status, 1)
        self.assertEqual(len(calls), 1)


if __name__ == '__main__':
    unittest.main()

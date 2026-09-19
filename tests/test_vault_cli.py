"""Integration tests against the compiled vault CLI; never mutate the real vault."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

SKILLS = Path(__file__).resolve().parents[1]
VAULT_BIN = SKILLS / 'vault-operator/scripts/vault-cli/target/release/vault'
CHECK_BIN = SKILLS / 'obsidian-markdown/scripts/vault-check/target/release/vault-check'

@unittest.skipUnless(VAULT_BIN.is_file() and CHECK_BIN.is_file(), "build both Rust release binaries first")
class VaultCLI(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.binaries = tempfile.TemporaryDirectory(prefix='vault binaries ')
        cls.addClassCleanup(cls.binaries.cleanup)
        cls.bin = Path(cls.binaries.name) / 'vault'
        shutil.copy2(VAULT_BIN, cls.bin)
        shutil.copy2(CHECK_BIN, Path(cls.binaries.name) / 'vault-check')

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='vault cli ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / '.obsidian').mkdir()
        (self.root / 'Topic').mkdir()
        self.git('init', '-q')
        self.git('config', 'user.email', 'test@example.invalid')
        self.git('config', 'user.name', 'Test')

    def git(self, *args):
        return subprocess.run(['git', '-C', str(self.root), *args], check=True, capture_output=True)

    def note(self, name, text):
        p = self.root / name
        p.write_text(text)
        return p

    def run_cli(self, *args, code=0, cwd=None):
        env = dict(os.environ)
        env.pop('VAULT_ROOT', None)
        p = subprocess.run([str(self.bin), *args], cwd=cwd or self.root, env=env, capture_output=True, text=True)
        self.assertEqual(p.returncode, code, p.stdout + p.stderr)
        return p.stdout

    def test_ranking_alias_content_and_open_uri(self):
        self.note('Entropy.md', '# Entropy\n')
        self.note('Topic/Second Law.md', '---\naliases: [Entropy]\n---\n')
        self.note('Other.md', 'Entropy is mentioned here.\n')
        self.assertEqual(self.run_cli('find', 'entropy', '--paths').splitlines(), ['Entropy.md', 'Topic/Second Law.md', 'Other.md'])
        url = self.run_cli('find', 'entropy', '--uri', '2')
        self.assertIn('Second%20Law.md', url)
        self.assertIn('obsidian://open?path=', url)
        self.run_cli('find', 'absent', code=1)
        self.run_cli('find', 'entropy', '--open', '0', code=2)

    def test_recent_hidden_and_nested_root(self):
        a = self.note('Old.md', '')
        b = self.note('Topic/New.md', '')
        self.note('.obsidian/Hidden.md', '')
        os.utime(a, (100, 100)); os.utime(b, (200, 200))
        self.assertEqual(self.run_cli('recent', '--paths', '--limit', '1', cwd=self.root/'Topic').strip(), 'Topic/New.md')

    def test_changed_includes_staged_unstaged_untracked_and_rename(self):
        self.note('Old.md', '# Old\n'); self.note('Edit.md', '# Edit\n')
        self.git('add', '.'); self.git('commit', '-qm', 'baseline')
        self.git('mv', 'Old.md', 'Renamed Note.md')
        self.note('Edit.md', '# Edit\n![[missing-edit.png]]\n')
        self.note('New Note.md', '![[missing-new.png]]\n')
        out = self.run_cli('check', '--changed', code=1, cwd=self.root/'Topic')
        self.assertIn('Checked 3 note(s)', out)
        self.assertIn('missing-edit.png', out); self.assertIn('missing-new.png', out)

    def test_embed_files_and_code_examples(self):
        self.note('Topic/Target.md', '# Target\n')
        (self.root/'Topic/pic one.png').write_bytes(b'fake')
        note = self.note('Topic/Note.md', '![[Target#Heading]]\n![x](pic%20one.png)\n`![[absent]]`\n```md\n![[absent]]\n```\n![](https://example.com/p.png)\n')
        self.run_cli('check', '--', str(note))
        note.write_text('![[lost.pdf#page=2]]\n')
        self.assertIn('missing embed: lost.pdf', self.run_cli('check', '--quiet', '--', str(note), code=1))

    def test_empty_changed_and_invalid_syntax(self):
        self.run_cli('check', '--changed')
        self.note('Bad.md', '---\naliases: [\n---\n')
        self.run_cli('check', '--changed', code=1)
        self.run_cli('check', code=2)
        self.run_cli('recent', '--nonsense', code=2)

if __name__ == '__main__':
    unittest.main()

"""Run with VALIDATE_PACKAGE and SOLVE_PACKAGE pointing to built binaries."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import subprocess

spec = importlib.util.spec_from_file_location('pack', Path(__file__).with_name('pack.py'))
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)
FIXTURES = Path(__file__).resolve().parents[1] / 'tests/fixtures'
VALIDATOR = os.environ['VALIDATE_PACKAGE']
SOLVER = os.environ['SOLVE_PACKAGE']

class Authoring(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.source = self.root/'source'
        self.source.mkdir()
        with zipfile.ZipFile(FIXTURES/'maintenance.orcpkg') as archive:
            for name in archive.namelist():
                target = self.source/name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.read(name))
        self.output = self.root/'result.orcpkg'
    def tearDown(self): self.temp.cleanup()
    def reject(self):
        with self.assertRaises((ValueError, OSError)):
            pack.pack(self.source, self.output, VALIDATOR)
        self.assertFalse(self.output.exists())
    def test_deterministic_and_existing_results(self):
        pack.pack(self.source, self.output, VALIDATOR)
        for path in self.source.rglob('*.xml'): os.utime(path, (999999999, 999999999))
        second = self.root/'second.orcpkg'
        pack.pack(self.source, second, VALIDATOR)
        self.assertEqual(self.output.read_bytes(), second.read_bytes())
        for mode in ('cost', 'delay'):
            output = subprocess.check_output([SOLVER, str(second), str(FIXTURES/'inputs/maintenance.json'), mode])
            self.assertEqual(json.loads(output)['solution'], json.loads((FIXTURES/f'expected/{mode}.json').read_text()))
        with self.assertRaises(FileExistsError): pack.pack(self.source, second, VALIDATOR)
    def test_xml_declaration_compatibility(self):
        path = self.source/'recipe.xml'; original = path.read_bytes()
        for declaration in (b'<?xml version="1.0"?>',
                            b"<?xml version='1.0' encoding='UTF-8'?>\n",
                            b'<?xml version="1.0" encoding="utf-8" standalone="yes"?>',
                            b'\xef\xbb\xbf<?xml version="1.0" encoding="UTF-8"?>'):
            path.write_bytes(declaration + original)
            pack.pack(self.source, self.output, VALIDATOR)
            self.output.unlink()
        for declaration in (b'<?xml version="1.0" encoding="UTF-16"?>',
                            b'<?xml version="1.0" encoding="ISO-8859-1"?>',
                            b'<?xml version="1.1"?>',
                            b' <?xml version="1.0"?>',
                            b'<?xml version="1.0"?><!DOCTYPE x>',
                            b'<?xml version="1.0"?><?other instruction?>',
                            b'<?xml version="1.0"?><?xml version="1.0"?>',
                            b'<?xml version="1.0?>', b'\xff'):
            path.write_bytes(declaration + original)
            self.reject()
        path.write_bytes(original.decode().encode('utf-16'))
        self.reject()

    def test_refresh_digest_and_changed_objective(self):
        model = self.source/'models/maintenance-cost.xml'
        model.write_bytes(model.read_bytes().replace(b'field="price"/>', b'field="delay"/>'))
        pack.pack(self.source, self.output, VALIDATOR)
        output = subprocess.check_output([SOLVER, str(self.output), str(FIXTURES/'inputs/maintenance.json'), 'cost'])
        self.assertEqual(json.loads(output)['solution'], json.loads((FIXTURES/'expected/delay.json').read_text()))
    def test_hidden_and_undeclared(self):
        for name in ('.hidden', 'extra.xml', 'MODEL.xml', 'notes.txt'):
            path = self.source/name; path.write_text('private')
            self.reject(); path.unlink()
    def test_symlinks(self):
        (self.source/'link.xml').symlink_to(self.source/'recipe.xml'); self.reject()
    def test_missing_duplicate_and_unsafe_manifest(self):
        path = self.source/'recipe.xml'; original = path.read_text()
        for changed in (original.replace('models/maintenance-delay.xml', 'models/missing.xml'),
                        original.replace('models/maintenance-delay.xml', 'models/maintenance-cost.xml'),
                        '<!DOCTYPE x [<!ENTITY a "x">]>'+original,
                        original.replace('tier="free"', 'tier="entitled"')):
            path.write_text(changed); self.reject()
        path.write_text(original)
    def test_bound_and_invalid_model(self):
        path = self.source/'models/maintenance-cost.xml'
        for content in (b'x'*32769, b'<descriptive/>'):
            path.write_bytes(content); self.reject()
    def test_hard_link_failure_no_publication(self):
        with patch.object(pack.os, 'link', side_effect=OSError('Hard links unsupported')):
            with self.assertRaises(OSError): pack.pack(self.source, self.output, VALIDATOR)
        self.assertFalse(self.output.exists())
        self.assertFalse(list(self.root.glob('.orcpkg-*')))

    def test_reject_validator_failure_no_output(self):
        with self.assertRaises(OSError): pack.pack(self.source, self.output, self.root/'missing-validator')
        self.assertFalse(self.output.exists())
        self.assertFalse(list(self.root.glob('.orcpkg-*')))

if __name__ == '__main__': unittest.main()

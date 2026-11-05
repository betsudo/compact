# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""PR license scope controls; the original full-header validator remains strict."""
from pathlib import Path
import hashlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_header_regressions as gate


class HeaderRegressionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(['git', 'init', '-q', self.root], check=True)
        self.path = self.root / 'source.py'
        self.path.write_text('print("existing debt")\n')
        subprocess.run(['git', 'add', 'source.py'], cwd=self.root, check=True)
        subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                        '-c', 'commit.gpgsign=false', 'commit', '-qm', 'fixture'], cwd=self.root, check=True)
        self.base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.root, text=True).strip()

    def test_actual_base_content_is_inherited_only_when_identical(self):
        result = gate.classify(self.root, self.base, [self.path])
        self.assertTrue(result['passed'])
        self.assertEqual(len(result['inherited_unchanged']), 1)
        self.path.write_text('print("modified debt")\n')
        result = gate.classify(self.root, self.base, [self.path])
        self.assertFalse(result['passed'])
        self.assertEqual(len(result['new_or_modified_violations']), 1)

    def test_new_file_and_rename_without_header_are_violations(self):
        new = self.root / 'new.py'
        new.write_bytes(self.path.read_bytes())
        result = gate.classify(self.root, self.base, [new])
        self.assertFalse(result['passed'])
        self.assertEqual(gate.base_bytes(self.root, self.base, 'new.py'), None)

    def test_batch_handles_duplicate_blobs_and_unusual_filenames(self):
        unusual = self.root / 'spaces and\nnewline.py'
        unusual.write_bytes(self.path.read_bytes())
        subprocess.run(['git', 'add', '--all'], cwd=self.root, check=True)
        subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                        '-c', 'commit.gpgsign=false', 'commit', '-qm', 'fixture names'], cwd=self.root, check=True)
        base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.root, text=True).strip()
        values = gate.base_contents(self.root, base, ['source.py', unusual.name, 'missing.py'])
        self.assertEqual(values['source.py'], values[unusual.name])
        self.assertIsNone(values['missing.py'])
        self.assertTrue(gate.classify(self.root, base, [self.path, unusual])['passed'])

    def test_invalid_base_is_not_a_missing_file(self):
        with self.assertRaises(subprocess.CalledProcessError):
            gate.base_bytes(self.root, '0' * 40, 'source.py')

    def test_all_missing_files_accounted_without_timestamp_trust(self):
        another = self.root / 'another.py'
        another.write_text('new')
        result = gate.classify(self.root, self.base, [self.path, another])
        self.assertEqual(result['missing_full_headers'], 2)
        self.assertEqual(len(result['inherited_unchanged']) + len(result['new_or_modified_violations']), 2)
        self.assertIn('not full-tree', result['scope'])

    def test_full_header_policy_still_rejects_short_spdx(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        short = '# This file is part of Compact.\n# Copyright (C) 2026 Midnight Foundation\n# SPDX-License-Identifier: Apache-2.0\n'
        self.assertFalse(manager._scan_for_header('#', short))
        self.assertTrue(manager._scan_for_header('#', manager._format_license_header('#', '2026')))


class PinnedHistoricalCopyTests(unittest.TestCase):
    def setUp(self):
        HeaderRegressionTests.setUp(self)
        self.copy_name, (self.authority_name, self.expected) = next(iter(gate.RETAINED_SOURCE_COPIES.items()))
        raw = (gate.ROOT / (self.authority_name + '.raw')).read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), self.expected)
        self.authority = self.root / self.authority_name
        self.authority.parent.mkdir(parents=True)
        self.authority.write_bytes(raw)
        subprocess.run(['git', 'add', self.authority_name], cwd=self.root, check=True)
        subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                        '-c', 'commit.gpgsign=false', 'commit', '-qm', 'original authority'], cwd=self.root, check=True)
        self.base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.root, text=True).strip()
        self.copy = self.root / self.copy_name
        self.copy.parent.mkdir(parents=True)
        self.copy.write_bytes(raw)

    def test_only_exact_known_original_copy_is_separately_accounted(self):
        result = gate.classify(self.root, self.base, [self.copy])
        self.assertTrue(result['passed'])
        self.assertEqual(result['inherited_unchanged'], [])
        self.assertEqual(len(result['retained_source_copies']), 1)
        self.assertEqual(result['retained_source_copies'][0]['sha256'], self.expected)
        self.assertEqual(result['retained_source_copies'][0]['authority'], self.authority_name)

    def test_changed_copy_or_authority_or_unknown_base_still_fails(self):
        for target in [self.copy, self.authority]:
            raw = target.read_bytes()
            target.write_bytes(raw + b' ')
            self.assertFalse(gate.classify(self.root, self.base, [self.copy])['passed'])
            target.write_bytes(raw)
        initial = subprocess.check_output(['git', 'rev-list', '--max-parents=0', 'HEAD'], cwd=self.root, text=True).strip()
        self.assertFalse(gate.classify(self.root, initial, [self.copy])['passed'])

    def test_a_new_alias_is_not_a_general_missing_header_exemption(self):
        alias = self.root / 'another.compact'
        alias.write_bytes(self.copy.read_bytes())
        self.assertFalse(gate.classify(self.root, self.base, [alias])['passed'])
        self.copy.unlink()
        self.copy.symlink_to(self.authority)
        self.assertFalse(gate.classify(self.root, self.base, [self.copy])['passed'])


    def test_authority_symlink_and_coordinated_nonliteral_bytes_refuse(self):
        raw = self.authority.read_bytes()
        target = self.root / 'original-target.compact'
        target.write_bytes(raw)
        self.authority.unlink()
        self.authority.symlink_to(target)
        result = gate.classify(self.root, self.base, [self.copy])
        self.assertFalse(result['passed'])
        self.assertEqual(result['retained_source_copies'], [])
        self.authority.unlink()
        self.authority.write_bytes(raw + b' ')
        self.copy.write_bytes(raw + b' ')
        result = gate.classify(self.root, self.base, [self.copy])
        self.assertFalse(result['passed'])
        self.assertEqual(result['retained_source_copies'], [])


class RustHeaderPlacementTests(unittest.TestCase):
    def setUp(self):
        HeaderRegressionTests.setUp(self)
        self.manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        self.header = self.manager._format_license_header('//', '2026') + '\n'

    def source(self, name, content):
        path = self.root / name
        path.write_bytes(content if isinstance(content, bytes) else content.encode())
        subprocess.run(['git', 'add', '--', name], cwd=self.root, check=True)
        return path

    def violations(self):
        return gate.rust_header_placement_violations(self.root, self.base, self.manager)

    def commit_fixture(self):
        subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                        '-c', 'commit.gpgsign=false', 'commit', '-qm', 'fixture'],
                       cwd=self.root, check=True)
        self.base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.root, text=True).strip()

    def test_full_header_precedes_module_documentation_and_code(self):
        body = '//! Module docs.\npub fn example() {}\n'
        path = self.source('module.rs', self.header + body)
        self.assertEqual(self.violations(), [])
        path.write_text(body + self.header)
        self.assertEqual(self.violations(), [path])

    def test_short_spdx_or_header_literal_is_not_a_leading_full_header(self):
        for index, content in enumerate([
                '// SPDX-License-Identifier: Apache-2.0\npub fn example() {}\n',
                'const TEXT: &str = r#"\n' + self.header + '"#;\n']):
            path = self.source(f'bad-{index}.rs', content)
            self.assertIn(path, self.violations())

    def test_inner_attributes_with_whitespace_and_nested_comments_are_code(self):
        for index, attribute in enumerate([
                '#![allow(dead_code)]', '#! [allow(dead_code)]', '#!\t[allow(dead_code)]',
                '#!\n[allow(dead_code)]', '#!/*c*/[allow(dead_code)]',
                '#! /* outer /* nested */ end */ [allow(dead_code)]',
                '#!\n// comment\n[allow(dead_code)]', '#!/* unterminated']):
            with self.subTest(attribute=attribute):
                path = self.source(f'attribute-{index}.rs', attribute + '\n' + self.header)
                self.assertIn(path, self.violations())

    def test_rust_script_shebang_and_utf8_bom_keep_leading_header(self):
        for index, preamble in enumerate(['#!/usr/bin/env rust-script\n', '#!rust-script\n', '\ufeff']):
            path = self.source(f'valid-{index}.rs', preamble + '\n' + self.header + 'fn main() {}\n')
            self.assertNotIn(path, self.violations())
        raw = (self.header + 'fn main() {}\n').replace('\n', '\r\n').encode()
        path = self.source('valid-crlf.rs', raw)
        self.assertNotIn(path, self.violations())
        self.assertEqual(path.read_bytes(), raw)
        path = self.source('invalid-utf8.rs', b'\xff' + self.header.encode())
        self.assertIn(path, self.violations())

    def test_other_languages_keep_original_header_rules(self):
        for name in ['counter_failing.compact', 'map_failing.compact', 'map_field_field.compact']:
            raw = (gate.ROOT / 'examples/adt/negative' / name).read_bytes()
            self.source(name, raw)
            self.assertTrue(self.manager._scan_for_header('//', raw.decode()))
        self.source('Dockerfile', '# syntax=docker/dockerfile:1\n')
        self.source('encoding.py', '# coding: latin-1\n')
        self.assertEqual(self.violations(), [])

    def test_unchanged_historical_generator_bytes_stay_unchanged_not_certified_as_a_header(self):
        authority = gate.ROOT / ('tools/compact-rust-backend/tests/retained-conformance-6e37443b/'
                                 'tools/compact-rust-backend/src/lib.rs')
        original = authority.read_bytes()
        retained = self.source('historical.rs', original)
        self.commit_fixture()
        self.assertEqual(self.violations(), [])
        # Its embedded GENERATED_HEADER is data, not this source file's license.
        copied = self.source('new-copy.rs', original)
        self.assertEqual(self.violations(), [copied])
        retained.write_bytes(original + b'\n// changed\n')
        self.assertIn(retained, self.violations())
        self.assertEqual(authority.read_bytes(), original)

    def test_unusual_filename_deletion_and_symlink_are_unambiguous(self):
        path = self.source('space and\nnewline.rs', self.header + 'fn main() {}\n')
        self.commit_fixture()
        path.write_text('fn main() {}\n' + self.header)
        self.assertEqual(self.violations(), [path])
        path.unlink()
        subprocess.run(['git', 'add', '--all'], cwd=self.root, check=True)
        self.assertEqual(self.violations(), [])
        link = self.root / 'link.rs'
        link.symlink_to(self.path)
        subprocess.run(['git', 'add', 'link.rs'], cwd=self.root, check=True)
        with self.assertRaisesRegex(ValueError, 'regular file'):
            self.violations()

    def test_main_deduplicates_missing_and_misplaced_rust_header(self):
        self.source('bad.rs', 'fn main() {}\n')
        (self.root / 'header_config.json').write_bytes((gate.ROOT / 'header_config.json').read_bytes())
        report = self.root / 'report.json'
        with patch.object(gate, 'ROOT', self.root):
            self.assertEqual(gate.main(['--base', self.base, '--report', str(report)]), 1)
        import json
        result = json.loads(report.read_text())
        self.assertEqual(len(result['new_or_modified_violations']), 1)
        self.assertEqual(result['rust_placement_violations'], ['bad.rs'])


if __name__ == '__main__':
    unittest.main()

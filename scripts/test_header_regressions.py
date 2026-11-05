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

    def test_full_header_must_precede_code_and_module_documentation(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        for prefix, preamble in [('#', 'print("code")\n'),
                                 ('#', '"""Module documentation."""\n'),
                                 ('//', 'use std::collections::HashMap;\n'),
                                 ('//', '//! Module documentation.\n'),
                                 ('//', '#![allow(dead_code)]\n')]:
            with self.subTest(prefix=prefix, preamble=preamble):
                header = manager._format_license_header(prefix, '2026') + '\n'
                self.assertTrue(manager._scan_for_header(prefix, header + preamble))
                self.assertFalse(manager._scan_for_header(prefix, preamble + header))

    def test_full_header_accepts_blank_lines_and_preserved_shebangs(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        header = manager._format_license_header('#', '2026') + '\n'
        for preamble in ['', '\n\n', '#!/usr/bin/env python3\n\n',
                         '#!/usr/bin/env nix-shell\n#! nix-shell -i bash\n\n']:
            with self.subTest(preamble=preamble):
                self.assertTrue(manager._scan_for_header('#', preamble + header + 'print(1)\n'))
        self.assertFalse(manager._scan_for_header('#', '#!/usr/bin/env python3\nprint(1)\n' + header))

    def test_full_header_preserves_chez_reader_preamble_not_executable_forms(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        header = manager._format_license_header(';;;', '2026') + '\n'
        self.assertTrue(manager._scan_for_header(';;;', '#!chezscheme\n\n' + header + '(library ())\n'))
        self.assertFalse(manager._scan_for_header(';;;', '#!chezscheme\n(library ())\n' + header))

    def test_slash_comment_scripts_keep_shebang_before_header_not_rust_attributes(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        header = manager._format_license_header('//', '2026') + '\n'
        for command in ['#!/usr/bin/env node', '#!/usr/bin/env rust-script']:
            with self.subTest(command=command):
                self.assertTrue(manager._scan_for_header('//', command + '\n\n' + header))
        self.assertEqual(manager._get_insert_index(['#![allow(dead_code)]\n']), 0)

    def test_footer_violation_is_not_inherited_after_modifying_base_code(self):
        manager = gate.HeaderManager(str(gate.ROOT / 'header_config.json'), str(self.root))
        header = manager._format_license_header('#', '2026') + '\n'
        self.path.write_text('print("modified debt")\n' + header)
        self.assertIn(self.path, manager.validate_headers())
        self.assertFalse(gate.classify(self.root, self.base, [self.path])['passed'])


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


if __name__ == '__main__':
    unittest.main()

#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Check the complete pinned reserved-keyword roster and diagnostic drift guard."""

import unittest

import check_upstream_ledger9_reserved_keywords as gate


class ReservedKeywordsGateTests(unittest.TestCase):
    def test_complete_original_case_roster(self):
        files = gate.original_inventory()
        tests = [name for name in files if name.endswith('/compile.fail.test.ts')]
        self.assertEqual(len(files), 96)
        self.assertEqual(len(tests), 48)
        self.assertEqual(len({name.rsplit('/', 1)[0] for name in tests}), 48)
        self.assertEqual({name.split('/')[3] for name in tests}, {'current', 'future'})

    def test_each_original_regex_rejects_changed_diagnostic(self):
        files = gate.original_inventory()
        for name in (name for name in files if name.endswith('/compile.fail.test.ts')):
            with self.subTest(name=name):
                pattern = gate.original_pattern(gate.ROOT / name)
                diagnostic = pattern.replace(r'\s+', '  ').replace(r'\s', ' ')
                # Every original asserts the parser category, so this mutation
                # must fail even when its location and keyword remain intact.
                self.assertFalse(gate.diagnostic_matches(pattern,
                                 diagnostic.replace('parse error:', 'changed diagnostic:')))


if __name__ == '__main__':
    unittest.main()

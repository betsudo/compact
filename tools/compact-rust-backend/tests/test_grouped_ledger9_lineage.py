#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""The grouped proof gates use reachable source lineage and immutable old receipts."""

import subprocess
import sys
from pathlib import Path
import unittest


GATES = Path(__file__).resolve().parents[1]
ROOT = GATES.parents[1]
sys.path.insert(0, str(GATES))

from check_upstream_ledger9_bytes_map_proof import (  # noqa: E402
    GROUPED_BASE, require_grouped_lineage,
)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


class GroupedLedger9LineageTest(unittest.TestCase):
    def test_current_source_descends_from_grouped_base(self):
        require_grouped_lineage(git('rev-parse', 'HEAD'))

    def test_source_before_grouped_base_is_refused(self):
        parent = git('rev-parse', f'{GROUPED_BASE}^')
        with self.assertRaisesRegex(RuntimeError, 'grouped collection base'):
            require_grouped_lineage(parent)


if __name__ == '__main__':
    unittest.main()

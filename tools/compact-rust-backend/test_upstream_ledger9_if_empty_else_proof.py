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

"""Focused physical-input guards for the original empty-else proof gate."""

from pathlib import Path
import tempfile
import unittest

from check_upstream_ledger9_if_empty_else_proof import (
    dependency_refusals, physical_sha256,
)


class ProofInputClosureTests(unittest.TestCase):
    def test_physical_digest_accepts_depfile_string(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / 'source.rs'
            source.write_text('fn main() {}\n')
            self.assertEqual(physical_sha256(str(source)), physical_sha256(source))

    def test_unknown_and_tampered_inputs_refused(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / 'source.rs'
            source.write_text('fn main() {}\n')
            tracked = {str(source)}
            inventory = {'source_hashes': {str(source): physical_sha256(source)}}
            self.assertEqual(
                dependency_refusals(tracked, inventory, root),
                {'unknown_dependency_rejected': True,
                 'tampered_dependency_hash_rejected': True},
            )


if __name__ == '__main__':
    unittest.main()

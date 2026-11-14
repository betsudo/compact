#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Admission checks for generated contracts consumed by funded collection gates."""

import sys
from pathlib import Path
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from collection_gate_binding import generated_contract_file_set


EXPECTED_FILES = (
    'Cargo.toml', 'compact-rust-ir.json', 'compact-rust-origins.raw.json',
    'index.d.ts', 'index.js', 'index.js.map', 'lib.rs', 'rust-capabilities.json',
)


class GeneratedContractFileSetTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.contract = self.root / 'contract'
        self.contract.mkdir()
        for name in EXPECTED_FILES:
            (self.contract / name).write_text(f'generated {name}\n')

    def test_accepts_exact_generated_file_set(self) -> None:
        generated_contract_file_set(self.contract)

    def test_rejects_added_cargo_build_script(self) -> None:
        (self.contract / 'build.rs').write_text('fn main() { panic!("injected"); }\n')
        with self.assertRaisesRegex(RuntimeError, 'unexpected file'):
            generated_contract_file_set(self.contract)

    def test_rejects_symlinked_expected_source(self) -> None:
        source = self.contract / 'lib.rs'
        source.unlink()
        outside = self.root / 'outside.rs'
        outside.write_text('pub fn injected() {}\n')
        source.symlink_to(outside)
        with self.assertRaisesRegex(RuntimeError, 'symlink'):
            generated_contract_file_set(self.contract)

    def test_rejects_symlinked_contract_directory(self) -> None:
        alias = self.root / 'contract-alias'
        alias.symlink_to(self.contract, target_is_directory=True)
        with self.assertRaisesRegex(RuntimeError, 'symlinked'):
            generated_contract_file_set(alias)


if __name__ == '__main__':
    unittest.main()

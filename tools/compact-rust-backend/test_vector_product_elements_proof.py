#!/usr/bin/env python3
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

"""Worktree depfile inputs can include prebound generated Cargo files."""

import tempfile
import unittest
from pathlib import Path

import check_vector_product_elements_proof as gate


class WorktreeInputBinding(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name) / "checkout"
        source = self.root / "tools/compact-rust-proof-smoke/src/main.rs"
        source.parent.mkdir(parents=True)
        source.write_text("fn main() {}\n")
        self.source = str(source)
        self.generated = []
        # The failed Linux depfile had 56 OUT_DIR contract.rs prerequisites
        # beneath the checkout's target directory.
        for index in range(56):
            path = (self.root / "target/proof-checks/build" /
                    f"fixture-{index:02d}/out/contract.rs")
            path.parent.mkdir(parents=True)
            path.write_text(f"pub const CASE: usize = {index};\n")
            self.generated.append(str(path))
        self.sources = {self.source: gate.digest(self.source)}
        self.build_inputs = {path: gate.digest(path) for path in self.generated}
        self.prerequisites = {self.source, *self.generated}

    def test_all_56_generated_worktree_inputs_retain_prebuild_binding(self):
        qualified, origins = gate.qualify_worktree_inputs(
            self.prerequisites, self.root, self.sources, self.build_inputs)
        self.assertEqual(qualified, self.sources | self.build_inputs)
        self.assertEqual(origins[self.source], ["source_hashes"])
        self.assertTrue(all(origins[path] == ["generated_build_inputs_before_compile"]
                            for path in self.generated))

    def test_existing_unknown_worktree_input_is_refused(self):
        unknown = self.root / "target/proof-checks/build/unknown/out/contract.rs"
        unknown.parent.mkdir(parents=True)
        unknown.write_text("unbound\n")
        with self.assertRaisesRegex(ValueError, "not pinned before build"):
            gate.qualify_worktree_inputs(self.prerequisites | {str(unknown)},
                                         self.root, self.sources, self.build_inputs)

    def test_changed_generated_input_is_refused(self):
        Path(self.generated[7]).write_text("altered\n")
        with self.assertRaisesRegex(ValueError, "changed after prebuild binding"):
            gate.qualify_worktree_inputs(self.prerequisites, self.root,
                                         self.sources, self.build_inputs)

    def test_conflicting_prebuild_authorities_are_refused(self):
        sources = self.sources | {self.generated[0]: "0" * 64}
        with self.assertRaisesRegex(ValueError, "not pinned before build"):
            gate.qualify_worktree_inputs(self.prerequisites, self.root,
                                         sources, self.build_inputs)


if __name__ == "__main__":
    unittest.main()

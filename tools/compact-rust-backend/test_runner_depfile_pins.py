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

"""A Cargo build output under the checkout is not a tracked source input."""
from pathlib import Path
import tempfile
import unittest

import local_parity_gate as gate


class RunnerDepfilePins(unittest.TestCase):
    def test_depfile_target_is_not_a_prerequisite(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = root / "src/lib.rs"
            source.parent.mkdir()
            source.write_text("source")
            raw = "target/proof-checks/compact-rust-proof-smoke: src/lib.rs\n"
            self.assertEqual(gate.depfile_prerequisites(raw, root), {str(source)})
            with self.assertRaises(FileNotFoundError):
                gate.depfile_prerequisites(raw.replace("src/lib.rs", "src/missing.rs"), root)

    def test_source_and_generated_target_have_distinct_prebuild_pins(self):
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory).resolve()
            source = root / "src/lib.rs"
            generated = root / "target/proof-checks/build/example/out/contract.rs"
            external = Path(outside).resolve() / "external-dependency"
            source.parent.mkdir()
            generated.parent.mkdir(parents=True)
            source.write_text("source")
            generated.write_text("generated")
            external.write_text("external")
            source_pins = {str(source): gate.sha256(source)}
            generated_pins = {str(generated): gate.sha256(generated)}
            tracked = {str(source), str(generated), str(external)}
            pinned_source, pinned_generated = gate.pin_runner_worktree_inputs(
                tracked, root, root / "target", source_pins, generated_pins)
            self.assertEqual(pinned_source, source_pins)
            self.assertEqual(pinned_generated, generated_pins)
            with self.assertRaises(gate.GateError):
                gate.pin_runner_worktree_inputs(tracked, root, root / "target", source_pins, {})
            generated.write_text("changed after prebuild pin")
            with self.assertRaises(gate.GateError):
                gate.pin_runner_worktree_inputs(tracked, root, root / "target", source_pins, generated_pins)


if __name__ == "__main__":
    unittest.main()

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

"""Refusal controls for the isolated SDK qualification boundary."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_functional_source_sdk as gate


class IsolatedSdkControls(unittest.TestCase):
    def test_extra_file_and_symlink_refuse_after_positive_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "lib.rs").write_text("raw SDK")
            expected = gate.inventory(root)
            gate.equal_inventory(root, expected)
            (root / "injected.rs").write_text("extra input")
            with self.assertRaisesRegex(ValueError, "membership"):
                gate.equal_inventory(root, expected)
            (root / "injected.rs").unlink()
            (root / "link").symlink_to(root / "lib.rs")
            with self.assertRaisesRegex(ValueError, "symlink"):
                gate.inventory(root)

    def test_library_recipe_preserves_original_and_refuses_ambiguous_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "original"
            (source / "src").mkdir(parents=True)
            (source / "src/lib.rs").write_text("original")
            original = b'[dependencies]\nruntime={path = "../runtime-rs"}\n'
            (source / "Cargo.toml").write_bytes(original)
            for name in ("README.md", "LICENSE"):
                (source / name).write_text(name)
            recipe = gate.source_testkit(source, root / "testkit-rs")
            self.assertEqual((root / "original-testkit-Cargo.toml").read_bytes(), original)
            self.assertEqual((source / "Cargo.toml").read_bytes(), original)
            self.assertEqual(recipe["transformation"]["count"], 1)
            self.assertNotEqual(recipe["original_sha256"], recipe["derived_sha256"])
            (source / "Cargo.toml").write_bytes(original * 2)
            with self.assertRaisesRegex(ValueError, "exactly once"):
                gate.source_testkit(source, root / "ambiguous")

    def test_dependency_graph_refuses_checkout_path_and_checksum_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            packages = []
            for name, relative in (("consumer", "consumer"), ("testkit", "testkit-rs"),
                                   ("contract", "generated/contract"),
                                   ("midnight-compact-runtime", "generated/contract/runtime-rs"),
                                   ("macros", "generated/contract/runtime-rs-macros")):
                packages.append({"name": name, "version": "0.0.0", "source": None,
                                 "manifest_path": str(root / relative / "Cargo.toml")})
            lock = root / "Cargo.lock"
            lock.write_text('[[package]]\nname="external"\nversion="1"\nsource="registry+test"\nchecksum="good"\n')
            cohort = {("external", "1", "registry+test"): "good"}
            gate.dependency_graph({"packages": packages}, lock, root, cohort)
            bad = copy.deepcopy(packages)
            bad[0]["manifest_path"] = str(root.parent / "checkout/Cargo.toml")
            with self.assertRaisesRegex(ValueError, "escapes"):
                gate.dependency_graph({"packages": bad}, lock, root, cohort)
            lock.write_text(lock.read_text().replace('"good"', '"bad"'))
            with self.assertRaisesRegex(ValueError, "checksum"):
                gate.dependency_graph({"packages": packages}, lock, root, cohort)

    def test_rust_error_needs_single_expected_code_and_intended_primary_span(self):
        path = Path("consumed_context_negative.rs")
        row = {"reason": "compiler-message", "message": {"level": "error", "code": {"code": "E0382"},
               "spans": [{"is_primary": True, "file_name": str(path), "line_start": 8, "line_end": 8}]}}
        def process(rows, status=101):
            return subprocess.CompletedProcess([], status, "\n".join(map(json.dumps, rows)), "")
        gate.typed_refusal(process([row]), "E0382", path, 8)
        for rows, code, line, status in (([row], "E0053", 8, 101), ([row], "E0382", 7, 101),
                                          ([row, row], "E0382", 8, 101), ([row], "E0382", 8, 0)):
            with self.assertRaises(ValueError):
                gate.typed_refusal(process(rows, status), code, path, line)

    def test_relaxed_missing_or_wrong_replay_results_cannot_pass(self):
        result = dict(status="passed", strict_application="default", replay_refusal="IntentAlreadyExists",
                      witness_private_state=[7, 8], skip_sequence=[None]*6,
                      standalone_proof_bytes=2912, transaction_bytes=6607)
        names = ("changed_binding_rejected", "changed_input_rejected", "changed_output_rejected",
                 "changed_private_witness_rejected", "changed_proof_rejected", "same_geometry_controls", "replay_unchanged")
        result.update({name: True for name in names})
        gate.validate_result(result)
        for name in names:
            bad = dict(result)
            bad.pop(name)
            with self.assertRaises(ValueError):
                gate.validate_result(bad)
        for key, value in (("strict_application", "relaxed"), ("replay_refusal", "any error"),
                           ("skip_sequence", [None, 1]), ("transaction_bytes", 0)):
            with self.assertRaises(ValueError):
                gate.validate_result(dict(result, **{key: value}))

    def test_selected_config_refuses_compiler_substitution_and_unchecked_profiles(self):
        for key in ("RUSTC_WRAPPER", "RUSTC_BOOTSTRAP", "CARGO_PROFILE_PROOF_CHECKS_DEBUG_ASSERTIONS",
                    "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS"):
            with self.assertRaisesRegex(ValueError, "override"):
                gate.cargo_configuration({key: "false"})
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text('[profile.proof-checks]\noverflow-checks=false\n')
            with patch.object(gate, "ROOT", root), self.assertRaisesRegex(ValueError, "profile"):
                gate.cargo_configuration({"CARGO_HOME": str(root / "cargo-home")})

    @unittest.skipUnless(os.name == "posix", "owned process group control")
    def test_actual_timeout_preserves_partial_output(self):
        with patch.object(gate, "TIMEOUT", 0.1):
            with self.assertRaises(subprocess.TimeoutExpired) as result:
                gate.bounded_command([sys.executable, "-c", "import time;print('started',flush=True);time.sleep(30)"], dict(os.environ))
        text = result.exception.stdout
        self.assertIn("started", text.decode() if isinstance(text, bytes) else text)


if __name__ == "__main__":
    unittest.main()

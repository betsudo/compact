#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Finite input-binding controls for Cargo proof runners."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from did_proof_gate import hashes
from proof_runner_input_binding import (depfile_prerequisites, prepare_generated_inputs,
                                        qualify_dependency_inputs)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class RunnerInputBindingTests(unittest.TestCase):
    def test_depfile_skips_target_and_resolves_escaped_prerequisites(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            first = root / "input one.rs"
            second = root / "other.rs"
            first.write_text("one\n")
            second.write_text("two\n")
            rule = "target/proof-checks/runner: input\\ one.rs \\\n other.rs\n"
            self.assertEqual(depfile_prerequisites(rule, root), {str(first), str(second)})
            with self.assertRaisesRegex(ValueError, "invalid actual runner dependency rule"):
                depfile_prerequisites("target/proof-checks/runner:\n", root)
            with self.assertRaises(FileNotFoundError):
                depfile_prerequisites("target/proof-checks/runner: missing.rs\n", root)

    def test_generated_target_input_is_prepared_and_source_owned(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            output = root / "evidence"
            output.mkdir()
            owner = root / "fixture/build.rs"
            owner.parent.mkdir()
            owner.write_text("fn main() {}\n")
            manifest = root / "fixture/Cargo.toml"
            manifest.write_text('[package]\nname = "fixture"\nversion = "1.0.0"\n')
            rerun = root / "fixture/source.txt"
            rerun.write_text("original source\n")
            (root / "Cargo.lock").write_text('[[package]]\nname = "fixture"\nversion = "1.0.0"\n')
            generated = root / "target/proof-checks/build/fixture/out/contract.rs"
            generated.parent.mkdir(parents=True)
            generated.write_text("pub const PINNED: u8 = 1;\n")
            script_log = generated.parent.parent / "output"
            script_log.write_text("cargo:rerun-if-changed=source.txt\n")
            package = {"id": "fixture 1.0.0", "name": "fixture", "version": "1.0.0",
                       "source": None, "manifest_path": str(manifest),
                       "targets": [{"kind": ["custom-build"], "src_path": str(owner)}]}
            report = {"source_hashes": hashes([owner, manifest, rerun, root / "Cargo.lock"])}
            calls = []

            def execute(label, command, stdout=None):
                calls.append((label, command))
                if label == "locked-build-metadata":
                    stdout.write_text(json.dumps({"packages": [package],
                                                  "target_directory": str(root / "target")}))
                elif label == "prepare-runner-inputs":
                    stdout.write_text(json.dumps({"reason": "build-script-executed",
                                                  "package_id": package["id"],
                                                  "out_dir": str(generated.parent)}) + "\n")

            prepare_generated_inputs(root, output, report, execute,
                                     "stateful-local-vector-widening", digest)
            self.assertEqual([call[0] for call in calls],
                             ["locked-build-metadata", "prepare-runner-inputs"])
            self.assertEqual(report["generated_build_inputs_before_compile"],
                             hashes([generated, script_log]))
            self.assertEqual(report["prepared_build_scripts"][0]
                             ["rerun_inputs_before_preparation"], hashes([rerun]))
            qualified, origins = qualify_dependency_inputs({str(owner), str(generated)},
                {"source_hashes": report["source_hashes"],
                 "generated_build_inputs_before_compile": report["generated_build_inputs_before_compile"]},
                digest)
            self.assertEqual(qualified[str(generated)], digest(generated))
            self.assertEqual(origins[str(generated)], ["generated_build_inputs_before_compile"])

            unknown = root / "target/proof-checks/build/fixture/out/unknown.rs"
            unknown.write_text("not pinned\n")
            with self.assertRaisesRegex(ValueError, "missing or inconsistent"):
                qualify_dependency_inputs({str(unknown)},
                    {"source_hashes": report["source_hashes"],
                     "generated_build_inputs_before_compile": report["generated_build_inputs_before_compile"]},
                    digest)
            generated.write_text("changed after preparation\n")
            with self.assertRaisesRegex(ValueError, "changed after physical prebuild binding"):
                qualify_dependency_inputs({str(generated)},
                    {"generated_build_inputs_before_compile": report["generated_build_inputs_before_compile"]},
                    digest)
            generated.write_text("pub const PINNED: u8 = 1;\n")
            rerun.write_text("changed after source pin\n")
            changed_report = {"source_hashes": report["source_hashes"].copy()}
            changed_output = root / "changed-evidence"
            changed_output.mkdir()
            with self.assertRaisesRegex(ValueError, "local generated-input source absent"):
                prepare_generated_inputs(root, changed_output, changed_report, execute,
                                         "stateful-product-vector-widening", digest)
            rerun.write_text("original source\n")
            untracked_report = {"source_hashes": hashes([owner, manifest, root / "Cargo.lock"])}
            untracked_output = root / "untracked-evidence"
            untracked_output.mkdir()
            with self.assertRaisesRegex(ValueError, "local generated-input source absent"):
                prepare_generated_inputs(root, untracked_output, untracked_report, execute,
                                         "stateful-product-vector-widening", digest)

    def test_conflicting_authorities_and_empty_depfile_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "input.rs"
            path.write_text("one\n")
            with self.assertRaisesRegex(ValueError, "empty actual runner dependency"):
                qualify_dependency_inputs(set(), {"source_hashes": hashes([path])}, digest)
            with self.assertRaisesRegex(ValueError, "missing or inconsistent"):
                qualify_dependency_inputs({str(path)},
                    {"source_hashes": hashes([path]), "generated": {str(path): "0" * 64}}, digest)


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Physical prerequisite regression: generated files may be inside a checkout."""
import tempfile
import unittest
from pathlib import Path

import check_vector_component_products_proof as gate


class PhysicalInputLocations(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve() / "checkout"
        self.files = []
        # These are different real producer roles, all physically under ROOT.
        for relative in ["src/main.rs", "ignored-evidence/sdk/contract/lib.rs",
                         "target/proof-checks/build/fixture/out/contract.rs"]:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(relative + "\n")
            self.files.append(str(path))
        self.inputs = {"source_hashes": {self.files[0]: gate.digest(self.files[0])},
                       "generated_input_hashes": {self.files[1]: gate.digest(self.files[1])},
                       "generated_build_inputs_before_compile": {self.files[2]: gate.digest(self.files[2])}}

    def actual_prerequisites(self, extra=()):
        raw = "runner: " + " ".join(self.files + list(extra)) + "\n"
        return gate.depfile_prerequisites(raw, self.root)

    def test_source_sdk_and_default_target_outputs_all_require_their_own_binding(self):
        qualified, origins = gate.qualify_dependency_inputs(self.actual_prerequisites(), self.inputs)
        self.assertEqual(set(qualified), set(self.files))
        self.assertEqual(origins[self.files[0]], ["source_hashes"])
        self.assertEqual(origins[self.files[1]], ["generated_input_hashes"])
        self.assertEqual(origins[self.files[2]], ["generated_build_inputs_before_compile"])

    def test_unknown_existing_target_input_is_never_skipped(self):
        unknown = self.root / "target/unbound.rs"
        unknown.write_text("present but not bound\n")
        with self.assertRaisesRegex(ValueError, "missing or inconsistent.*unbound.rs"):
            gate.qualify_dependency_inputs(self.actual_prerequisites([str(unknown)]), self.inputs)

    def test_missing_actual_prerequisite_refuses_before_classification(self):
        missing = str(self.root / "target/missing.rs")
        with self.assertRaises(FileNotFoundError):
            self.actual_prerequisites([missing])

    def test_each_generated_role_requires_its_prebuild_hash(self):
        for role in ["generated_input_hashes", "generated_build_inputs_before_compile"]:
            altered = {key: dict(value) for key, value in self.inputs.items()}
            altered[role] = {}
            with self.subTest(role=role), self.assertRaisesRegex(ValueError, "missing or inconsistent"):
                gate.qualify_dependency_inputs(self.actual_prerequisites(), altered)

    def test_changed_generated_bytes_cannot_be_blessed_by_their_location(self):
        Path(self.files[2]).write_text("changed OUT_DIR input\n")
        with self.assertRaisesRegex(ValueError, "changed after physical prebuild binding"):
            gate.qualify_dependency_inputs(self.actual_prerequisites(), self.inputs)

    def test_conflicting_prebuild_roles_refuse(self):
        self.inputs["generated_input_hashes"][self.files[2]] = "0" * 64
        with self.assertRaisesRegex(ValueError, "missing or inconsistent"):
            gate.qualify_dependency_inputs(self.actual_prerequisites(), self.inputs)


if __name__ == "__main__":
    unittest.main()

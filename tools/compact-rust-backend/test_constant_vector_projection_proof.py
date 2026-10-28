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

"""Source-bound Vector proof receipt and mutation refusal controls."""
import copy
import json
import re
import unittest

import check_constant_vector_projection_proof as gate


def completed(label="constant-valued-vector-indices"):
    return {"format": gate.PROOF_RESULT_FORMAT, "status": "passed", "source": label,
            "constructor_execution_proved": False,
            "calls": [{"select_first": first, "expected": 17 if first else 41,
                       "proof_bytes": 100, "changed_binding_rejected": True,
                       "changed_result_rejected": True, "changed_input_same_geometry": True,
                       "changed_input_rejected": True, "changed_proof_bytes_rejected": True,
                       "strict_application": "default", "replay_refusal": "IntentAlreadyExists",
                       "replay_unchanged": True} for first in [False, True]]}


class VectorProofProtocolTests(unittest.TestCase):
    def test_each_source_has_a_separate_whole_ir_and_fixture_binding(self):
        root = gate.ROOT / "tools/compact-rust-proof-smoke/src"
        pairs = [("constant_vector_projection.rs", "literal-vector-projection",
                  "compact_rust_constant_vector_projection_source_fixture",
                  "constant-vector-projection-source"),
                 ("constant_valued_vector_indices.rs", "constant-valued-vector-indices",
                  "compact_rust_constant_valued_vector_indices_source_fixture",
                  "constant-valued-vector-indices-source")]
        for name, label, fixture, directory in pairs:
            text = (root / name).read_text()
            self.assertIn(f'const SOURCE_LABEL: &str = "{label}";', text)
            self.assertIn(f"use {fixture} as fixture;", text)
            self.assertIn(f"{directory}/original-ir.json", text)
            self.assertIn('include!("vector_projection_protocol.rs")', text)
        protocol = (root / "vector_projection_protocol.rs").read_text()
        self.assertEqual(re.findall(r'const PROOF_RESULT_FORMAT: &str = "([^"\n]+)";', protocol),
                         [gate.PROOF_RESULT_FORMAT])
        self.assertRegex(protocol, r'"format"\s*:\s*PROOF_RESULT_FORMAT')

    def test_complete_current_receipts_accept_both_declared_sources(self):
        for label in ["literal-vector-projection", "constant-valued-vector-indices"]:
            gate.validate_proof_summary(completed(label), label)

    def test_another_source_or_old_format_is_not_accepted(self):
        for field, value in [("format", "unknown/v1"), ("source", "literal-vector-projection"),
                             ("status", "incomplete"), ("constructor_execution_proved", True)]:
            row = completed(); row[field] = value
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(row, "constant-valued-vector-indices")

    def test_pass_flag_cannot_replace_actual_required_crypto_application_controls(self):
        fields = ["changed_binding_rejected", "changed_result_rejected", "changed_input_same_geometry",
                  "changed_input_rejected", "changed_proof_bytes_rejected", "replay_unchanged"]
        for field in fields:
            for value in [False, None, 1]:
                row = completed(); row["calls"][0][field] = value
                with self.subTest(field=field, value=value), self.assertRaises(gate.common.GateError):
                    gate.validate_proof_summary(row, "constant-valued-vector-indices")
        for field, value in [("proof_bytes", 0), ("proof_bytes", True), ("expected", True),
                             ("expected", 17), ("select_first", 0), ("strict_application", "weak"),
                             ("replay_refusal", "other")]:
            row = completed(); row["calls"][0][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(row, "constant-valued-vector-indices")

    def test_missing_reordered_or_malformed_branches_refuse(self):
        examples = [None, {}, {"calls": None}]
        row = completed(); row["calls"].pop(); examples.append(row)
        row = completed(); row["calls"].reverse(); examples.append(row)
        row = completed(); row["calls"][0] = None; examples.append(row)
        for row in examples:
            with self.subTest(row=row), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(row, "constant-valued-vector-indices")

    def test_index_control_changes_a_genuine_in_range_value_without_mutating_source(self):
        for directory in ["constant-vector-projection-source", "constant-valued-vector-indices-source"]:
            ir = json.loads((gate.ROOT / "tests-rust-backend" / directory / "original-ir.json").read_text())
            before = copy.deepcopy(ir)
            changed = gate.changed_projection_ir(ir)
            self.assertEqual(ir, before)
            self.assertNotEqual(changed, ir)
            first = next(row for row in changed["circuits"] if row["name"] == "first")
            node = first["body"]
            while node["kind"] == "let":
                node = node["body"]
            self.assertEqual(node["index"], 1)
            node["index"] = 0
            self.assertEqual(changed, ir)

    def test_index_control_refuses_missing_or_wrong_source_geometry(self):
        ir = json.loads((gate.FIXTURE / "original-ir.json").read_text())
        examples = [{"circuits": []}]
        for length in [1, 0, 3]:
            changed = copy.deepcopy(ir)
            first = next(row for row in changed["circuits"] if row["name"] == "first")
            first["parameters"][0]["ty"]["length"] = length
            examples.append(changed)
        for row in examples:
            with self.subTest(row=row), self.assertRaises(gate.common.GateError):
                gate.changed_projection_ir(row)


if __name__ == "__main__":
    unittest.main()

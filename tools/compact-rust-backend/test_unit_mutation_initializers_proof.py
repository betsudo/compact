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

"""Cheap native-producer/proof-consumer protocol and false-success controls."""
import copy
import re
import unittest
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import check_unit_mutation_initializers_proof as gate


def completed():
    return {"format": "compact-unit-mutation-initializers-proof/v1", "status": "passed",
            "constructor_execution_proved": False, "unselected_branch_proved": False,
            "prestate_setup_proved": False, "all_exports_proved": False,
            "calls": [{"key": key, "before": 1, "after": after, "proof_bytes": 100, "transaction_bytes": 120,
                       "standalone_proof":f"unit-initializers-{key-5}.proof",
                       "applied_transaction":f"unit-initializers-{key-5}-transaction.bin",
                       "transaction_proof_relationship":"independently proved",
                       "changed_result_same_geometry": True, "changed_binding_rejected": True,
                       "changed_result_rejected": True, "strict_application": "default",
                       "replay_refusal": "IntentAlreadyExists", "replay_unchanged": True}
                      for key, after in [(5, 1), (6, 2)]]}


class ProofReceiptTests(unittest.TestCase):
    def test_native_producer_uses_the_consumed_format(self):
        source = (gate.ROOT / "tools/compact-rust-proof-smoke/src/unit_mutation_initializers.rs").read_text()
        formats = re.findall(r'const PROOF_RESULT_FORMAT: &str = "([^"\n]+)";', source)
        self.assertEqual(formats, [gate.PROOF_RESULT_FORMAT])
        self.assertRegex(source, r'"format"\s*:\s*PROOF_RESULT_FORMAT')

    def test_complete_selected_protocol_is_accepted(self):
        gate.validate_proof_summary(completed())

    def test_old_or_unknown_receipt_format_is_not_success(self):
        for format_name in ["compact-field-wrappers-expression-proof/v1", "unknown/v1"]:
            data = completed()
            data["format"] = format_name
            with self.subTest(format=format_name), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)

    def test_pass_flag_does_not_replace_proof_verification_application_or_replay(self):
        for key, value in [("proof_bytes", 0), ("transaction_bytes",0),
                           ("transaction_proof_relationship","reused standalone proof"),
                           ("applied_transaction","unit-initializers-0.proof"),
                           ("standalone_proof","other-source.proof"), ("changed_result_same_geometry", False),
                           ("changed_binding_rejected", False), ("changed_result_rejected", False),
                           ("strict_application", "unbalanced"), ("replay_refusal", "other"),
                           ("replay_unchanged", False)]:
            data = completed()
            data["calls"][0][key] = value
            with self.subTest(field=key), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)

    def test_selected_calls_and_source_transition_are_required(self):
        cases = []
        data = completed(); data["calls"].pop(); cases.append(data)
        data = completed(); data["calls"].reverse(); cases.append(data)
        data = completed(); data["calls"][0]["after"] = 2; cases.append(data)
        data = completed(); data["calls"][1]["before"] = 0; cases.append(data)
        for data in cases:
            with self.subTest(data=data), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)

    def test_dirty_gate_entries_fail_before_generation(self):
        import check_unit_mutation_initializers as api
        for name in ["api", "proof"]:
            with self.subTest(gate=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                output = root / "output"
                report = root / "receipt.json" if name == "api" else output / "receipt.json"
                compiler = root / "compactc"
                compiler.write_bytes(b"must not execute")
                with patch.object(api.subprocess, "check_output",
                                  side_effect=["head\n", " M runtime-rs/src/lib.rs\n"]), patch.object(
                        api.subprocess, "run") as execute, patch.object(
                        api, "scheme_compiler", return_value=compiler):
                    with self.assertRaisesRegex(RuntimeError, "clean source checkout required"):
                        if name == "api":
                            with patch("sys.argv", ["gate", "--compiler", str(compiler),
                                       "--output", str(output), "--report", str(report),
                                       "--ts-runtime", str(root)]):
                                api.main()
                        else:
                            gate.run(SimpleNamespace(output=output, target_dir=root))
                    execute.assert_not_called()
                receipt = json.loads(report.read_text())
                self.assertEqual(receipt["status"], "failed")
                self.assertTrue(receipt["source_dirty"])
                self.assertEqual(receipt["error"], "clean source checkout required")

    def test_scope_claims_and_integer_types_are_exact(self):
        for field in ["constructor_execution_proved", "unselected_branch_proved", "prestate_setup_proved", "all_exports_proved"]:
            data = completed(); data[field] = True
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)
        for field, value in [("proof_bytes", True), ("transaction_bytes",True), ("before", True), ("key", 5.0), ("after", 1.0)]:
            data = copy.deepcopy(completed()); data["calls"][0][field] = value
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)


if __name__ == "__main__":
    unittest.main()

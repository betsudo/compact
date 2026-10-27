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

"""False-success controls for the selected DID native proof receipt."""
import unittest
from unittest.mock import patch
import tempfile
from pathlib import Path
from types import SimpleNamespace
import check_did_functional_lifecycle_proof as gate


def completed():
    return {"format":gate.PROOF_RESULT_FORMAT,"status":"passed",
            "constructor_execution_proved":False,"all_twelve_exports_proved":False,
            "execution":"functional-recorded","circuit":"deactivate","proof_bytes":100,
            "changed_binding_rejected":True,"changed_version_input_rejected":True,
            "changed_input_same_geometry":True,"replay_unchanged":True,"stale_version_rejected":True,
            "stale_version_error":"Controller authorization version is stale",
            "standalone_call_proof_file":"deactivate-input-control.proof",
            "applied_transaction_file":"did-functional-applied-transaction.bin",
            "applied_transaction_bytes":200,
            "proof_relationship":"standalone mutation-control proof; transaction independently proved and default-strictly verified",
            "strict_application":"default","replay_refusal":"IntentAlreadyExists",
            "changed_unit_result_control":"not applicable: Unit has zero output atoms","final_state_hex":"00"}


class ProofReceiptTests(unittest.TestCase):
    def test_complete_selected_receipt(self):
        gate.validate_proof_summary(completed())

    def test_result_protocol_rejects_incomplete_or_mistyped_fields(self):
        changes = [("applied_transaction_bytes",0),("applied_transaction_bytes",True),
                   ("stale_version_error","other"),("applied_transaction_file","other"),
                   ("proof_relationship","same proof"),("standalone_call_proof_file","other"),
                   ("proof_bytes",0),("proof_bytes",True),("changed_binding_rejected",False),
                   ("changed_version_input_rejected",False),("changed_input_same_geometry",False),
                   ("replay_unchanged",False),("stale_version_rejected",False),
                   ("strict_application","unbalanced"),("replay_refusal","other"),
                   ("final_state_hex",""),("changed_unit_result_control","result mutated")]
        for field, value in changes:
            data=completed(); data[field]=value
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)

    def test_scope_cannot_expand_or_use_another_execution_route(self):
        for field, value in [("format","unknown/v1"),("status","incomplete"),
                             ("constructor_execution_proved",True),("all_twelve_exports_proved",True),
                             ("execution","ordinary-recorded"),("circuit","constructor")]:
            data=completed(); data[field]=value
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_proof_summary(data)

    def test_dirty_source_is_refused_before_generation(self):
        with patch("check_did_functional_lifecycle.subprocess.check_output",
                   side_effect=["head\n", " M runtime-rs/src/lib.rs\n"]):
            receipt = {}
            with self.assertRaisesRegex(gate.common.GateError, "clean source checkout required"):
                gate.require_clean_source(receipt)
            self.assertTrue(receipt["source_dirty"])

    def test_proof_capture_must_equal_complete_ts_capture(self):
        import check_did_functional_lifecycle as api
        original = api.json.loads
        def changed(text):
            data = original(text)
            if "cases" in data and any(row.get("source") != "did" for row in data["cases"]):
                next(row for row in data["cases"] if row["id"] == "did/valid")["privateAfter"] = 999
            return data
        with patch.object(api.json, "loads", side_effect=changed):
            with self.assertRaisesRegex(gate.common.GateError, "captures differ"):
                gate.checked_oracle_captures()

    def test_failed_gate_writes_failed_receipt_and_returns_nonzero(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "gate"
            args = SimpleNamespace(output=output, target_dir=Path(directory))
            with patch.dict(gate.os.environ, {"MIDNIGHT_PP": directory}), patch.object(
                    gate, "require_clean_source", side_effect=gate.common.GateError("dirty control")):
                self.assertEqual(gate.run(args), 1)
            receipt = gate.json.loads((output / "receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["error"], "dirty control")


if __name__ == "__main__":
    unittest.main()

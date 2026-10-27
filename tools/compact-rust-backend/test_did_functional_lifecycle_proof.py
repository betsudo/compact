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
import check_did_functional_lifecycle_proof as gate


def completed():
    return {"format":gate.PROOF_RESULT_FORMAT,"status":"passed",
            "constructor_execution_proved":False,"all_twelve_exports_proved":False,
            "execution":"functional-recorded","circuit":"deactivate","proof_bytes":100,
            "changed_binding_rejected":True,"changed_version_input_rejected":True,
            "changed_input_same_geometry":True,"replay_unchanged":True,"stale_version_rejected":True,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists",
            "changed_unit_result_control":"not applicable: Unit has zero output atoms","final_state_hex":"00"}


class ProofReceiptTests(unittest.TestCase):
    def test_complete_selected_receipt(self):
        gate.validate_proof_summary(completed())

    def test_pass_flag_cannot_replace_proof_or_controls(self):
        changes = [("proof_bytes",0),("proof_bytes",True),("changed_binding_rejected",False),
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


if __name__ == "__main__":
    unittest.main()

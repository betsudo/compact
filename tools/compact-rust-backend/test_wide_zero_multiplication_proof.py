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

"""Reject incomplete or type-confused proof summaries; these are parser controls."""
import copy
import unittest
from check_wide_zero_multiplication_proof import validate_proof_summary
from local_parity_gate import GateError

class WideZeroProofSummaryTests(unittest.TestCase):
    def test_complete_routes_and_exact_primitive_types_are_required(self):
        row = dict(result=0, proof_bytes=2912, changed_binding_rejected=True,
                   changed_result_rejected=True, changed_input_same_geometry=True, changed_result_same_geometry=True,
                   changed_input_rejected=True, changed_proof_bytes_rejected=True,
                   replay_unchanged=True, strict_application="default", replay_refusal="IntentAlreadyExists")
        value = dict(format="compact-wide-zero-proof/v1", status="passed",
                     constructor_execution_proved=False,
                     calls=[dict(row, name=name) for name in ["multiplyRight", "multiplyLeft"]])
        validate_proof_summary(value)
        for field, changed in [("result",False),("result",1),("proof_bytes",True),
                               ("changed_result_same_geometry",False),("changed_result_same_geometry",1),
                               ("changed_input_rejected",False),("changed_input_rejected",1),
                               ("strict_application","relaxed"),("replay_refusal","other")]:
            with self.subTest(field=field,changed=changed):
                bad=copy.deepcopy(value);bad["calls"][0][field]=changed
                with self.assertRaises(GateError):validate_proof_summary(bad)
        for rows in [value["calls"][:1],value["calls"][::-1],value["calls"]*2]:
            bad=copy.deepcopy(value);bad["calls"]=rows
            with self.assertRaises(GateError):validate_proof_summary(bad)

if __name__=="__main__":unittest.main()

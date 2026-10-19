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

import copy
import unittest

import compare_forge_contract_loop as loop


class ComparisonTests(unittest.TestCase):
    def fixture(self):
        cases = []
        for name, config in loop.native.read(loop.harness.LARGE_QUALIFICATIONS).items():
            routes = {circuit: "forge-hit" for circuit in config["circuits"]}
            for circuit in ("observe", "forget", "reset"):
                if circuit in routes:
                    routes[circuit] = "auto-bypass"
            cases.append(dict(circuit=name, artifact_hashes_by_circuit={c: {"prover": "exact"} for c in config["circuits"]},
                publication_identity={"manifest_sha256": "exact"}, acquisition_paths=routes,
                proof_and_assertions={"exit_code": 0, "stdout": "\n".join(config["success_markers"])},
                corrupt_verifier_control={"exit_code": 1, "stderr": "expected header tag 'midnight:verifier-key[v6]:'"}))
        before = dict(requests=6, hits=0, misses=6, errors=0, generation_ms=50)
        return dict(passed=True, workload_set="ledger8-large", acquisition_accounting_complete=True, cases=cases,
                    stats_before={"totals": before}, stats_after={"totals": dict(before, requests=12, hits=6)})

    def test_complete_proof_and_route_controls_refuse_false_success(self):
        value = self.fixture()
        self.assertEqual(len(loop.identities(value, "forge-hit")), 2)
        for mutate in (lambda v: v["cases"][0]["proof_and_assertions"].update(exit_code=1),
                       lambda v: v["cases"][0]["proof_and_assertions"].update(stdout=""),
                       lambda v: v["cases"][0]["corrupt_verifier_control"].update(exit_code=0),
                       lambda v: v["cases"][0]["acquisition_paths"].update(acceptAttestation="forge-miss"),
                       lambda v: v["stats_after"]["totals"].update(generation_ms=51),
                       lambda v: v["stats_after"]["totals"].update(errors=1),
                       lambda v: v["cases"][0]["artifact_hashes_by_circuit"].pop("acceptAttestation")):
            changed = copy.deepcopy(value)
            mutate(changed)
            with self.assertRaises(ValueError):
                loop.identities(changed, "forge-hit")

    def test_cold_and_restored_are_not_in_the_warm_sample(self):
        rows = {name: {"whole_command_seconds": seconds} for name, seconds in
                (("ordinary-1", 11), ("ordinary-2", 9), ("warm-1", 8), ("warm-2", 10), ("cold", 100), ("restored", 50))}
        self.assertEqual(loop.comparison(rows), dict(median_ordinary_seconds=10, median_warm_seconds=9,
                                                   absolute_savings_seconds=1, relative_savings=0.1))


if __name__ == "__main__":
    unittest.main()

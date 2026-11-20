#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Negative evidence controls; injected commands never build or prove."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import acc_jubjub_gate as gate


def put(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value) if not isinstance(value, str) else value)


def capture():
    return {"format": "compact-acc-jubjub-wrapper-capture/v1", "kind": gate.ROW["kind"],
            "cases": [{"id": name, "operation": "apply"} for name in gate.CASES],
            "rawCases": [{"id": "raw-q_minus_one"}, {"id": "raw-q"}],
            "refusals": [{"id": name} for name in ("raw-q-generator", "raw-q-point", "negative",
                                                  "native-modulus", "number-not-bigint")]}


def summary():
    return {"format": "compact-did-primitive-reducer-proof/v2", "kind": gate.ROW["kind"],
            "status": "passed", "strictness": "default", "constructor_data_deployed": True,
            "constructor_execution_proved": False,
            "calls": [dict(row, changed_binding_rejected=True, applied=True, recorded_state_matches=True,
                           replay_unchanged=True, replay_refusal="IntentAlreadyExists", proof_bytes=5,
                           transaction_proof_bytes=8, transaction_proof_version=3, funded_dust_spends=1,
                           separate_verification=True, corrupt_proof_rejected=True,
                           state_bytes=5) for row in gate.ROW["proof_cases"]]}


class JubjubGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.target = self.root / "target"
        for obj, name, value in ((gate, "ROOT", self.root), (gate, "source_inventory", None),
                                 (gate.common, "git_head", None)):
            mock = patch.object(obj, name, value) if value else patch.object(
                obj, name, return_value={} if name == "source_inventory" else "frozen-test-head")
            mock.start()
            self.addCleanup(mock.stop)
        for name in ("compiler", "scheme", "tools/zkir-v3", "tools/node", "tools/rustfmt", "tools/rustup"):
            path = self.root / name
            put(path, "#!/bin/sh\nexit 0\n")
            path.chmod(0o755)
        for name in ("pp", "static"):
            (self.root / name).mkdir()
        dust = self.root / "static/dust/spend.bzkir"
        put(dust, "pinned Dust fixture")
        digest = gate.common.sha256(dust)
        put(dust.with_name("spend.bzkir.sha256"), digest)
        put(dust.with_name("spend.prover.sha256"), gate.did.ledger_static.PROVER_HASH)
        mock = patch.object(gate.did.ledger_static, "FIXTURE_HASHES", {"spend.bzkir": digest})
        mock.start()
        self.addCleanup(mock.stop)
        for name in (gate.SOURCE, gate.CAPTURE, gate.BOUNDARIES):
            put(self.root / name, "source")
        put(self.root / gate.FIXTURE, "fresh fixture")
        put(self.root / gate.IR, {"schema_version": 20})
        put(self.root / gate.REFERENCE, capture())
        put(self.root / gate.CURRENT_CAPTURE, capture())
        put(self.root / gate.RUNTIME_PACKAGE, {"name": "@midnight-ntwrk/compact-runtime", "version": "0.17.0"})
        self.node_library = self.root / "node/lib/libnode.dylib"
        put(self.node_library, "Node dynamic library")
        self.runtime = self.root / "runtime/dist/index.js"
        put(self.runtime, "runtime closure input")
        self.env = {"MIDNIGHT_PP": str(self.root / "pp"),
                    "MIDNIGHT_LEDGER_TEST_STATIC_DIR": str(self.root / "static"),
                    "PATH": str(self.root / "tools")}
        self.calls = []
        self.corrupt = None

    def command(self, argv, label, directory, receipt, *, env, stdout_path):
        self.calls.append(label)
        log = directory / "logs" / (label + ".log")
        keyshape = "(k=12, rows=2236)" if self.corrupt != "keyshape" else "(k=12, rows=2237)"
        put(log, keyshape if label == "keygen-apply" else "ok")
        receipt["commands"].append({"argv": argv, "label": label, "log": str(log), "exit_code": 0})
        output = directory / gate.ROW["kind"]
        if label == "zkir-version":
            put(stdout_path, "midnight-zkir 2.2.0" if self.corrupt == "zkir-version" else "midnight-zkir 3.2.0")
        elif label == "select-rustfmt":
            self.assertEqual(argv[1:], ["which", "--toolchain", "1.99.0", "rustfmt"])
            put(stdout_path, str(self.root / "tools/rustfmt"))
        elif label in ("freeze-node-runtime", "recheck-node-runtime"):
            self.assertEqual(Path(argv[0]), (self.root / "tools/node").resolve())
            self.assertEqual(argv[1:], ["-e", gate.NODE_SHARED_OBJECTS])
            put(stdout_path, {"sharedObjects": [str(self.root / "tools/node"), str(self.node_library),
                                               "/usr/lib/libSystem.B.dylib"]})
        elif label == "freeze-ts-runtime":
            self.assertEqual(argv[2:], ["--runtime-root", str(self.root.resolve() / "runtime"), "--runtime-inventory"])
            put(stdout_path, gate.hashes([self.runtime, self.root / gate.RUNTIME_PACKAGE]))
        elif label == "compile-jubjub":
            self.assertIn("--feature-zkir-v3", argv)
            put(output / "contract/lib.rs", "different" if self.corrupt == "fixture" else "fresh fixture")
            put(output / "contract/index.js", "generated JavaScript")
            rows = [{"name": name, "proof_required": True, "recorded": True,
                     "observed_call": True, "recording_status": "available"} for name in ("apply", "raw")]
            if self.corrupt == "capability":
                rows.pop()
            put(output / "contract/rust-capabilities.json", {"schema_version": 3, "circuits": rows})
            put(output / "compiler/contract-info.json", {"circuits": [{"name": name, "proof": True} for name in ("apply", "raw")]})
            put(output / "compiler/contract-manifest.json", "compiler manifest")
            put(output / "zkir/apply.zkir", "zkir")
            put(output / "zkir/raw.zkir", "zkir")
            put(output / "contract/compact-rust-ir.json", {"schema_version": 0 if self.corrupt == "ir" else 20})
        elif label == "compare-formatted-fixture":
            self.assertEqual(Path(argv[3]), directory / "formatted-fixture.rs")
            self.assertIsNone(stdout_path)
        elif label == "capture-ts-reference":
            self.assertIn("--ledger9", argv)
            self.assertEqual(argv[argv.index("--runtime-root") + 1], str(self.root.resolve() / "runtime"))
            self.assertIn("--reference", argv)
            self.assertNotIn("--oracle", argv)
            value = capture()
            if self.corrupt == "capture":
                value["cases"][0]["unexpected"] = "drift"
            put(stdout_path, value)
            files = [self.root / p for p in (gate.CAPTURE, gate.SOURCE, gate.REFERENCE, gate.BOUNDARIES)]
            files.extend([output / "contract/index.js", self.runtime])
            prov = {"format": "compact-acc-jubjub-wrapper-provenance/v1",
                    "oracleMode": "retained-original-reference", "runtime": {"path": str(self.runtime)},
                    "runtimeHashes": gate.hashes([self.runtime, self.root / gate.RUNTIME_PACKAGE]),
                    "inputs": [{"path": str(p), "sha256": gate.common.sha256(p)} for p in files]}
            if self.corrupt == "provenance":
                prov["oracleMode"] = "live-original"
            if self.corrupt == "missing-reference":
                prov["inputs"] = [p for p in prov["inputs"] if p["path"] != str(self.root / gate.REFERENCE)]
            put(directory / "capture-provenance.json", prov)
        elif label == "model-acc-jubjub-cell":
            put(log, 'circuit "apply" (k=12, rows=2236)\n'
                + ('circuit "raw" (k=10, rows=797)' if self.corrupt == "model" else 'circuit "raw" (k=10, rows=796)'))
        elif label == "test-jubjub-behavior":
            self.assertIn("-j4", argv)
            self.assertIn("--locked", argv)
            self.assertIn("--offline", argv)
            self.assertIn("--all-features", argv)
        elif label == "build-proof-runner":
            self.assertIn("proof-checks", argv)
            runner = self.target / "proof-checks/compact-rust-proof-smoke"
            put(runner, "#!/bin/sh\nexit 0\n")
            runner.chmod(0o755)
        elif label == "prepare-proof-material":
            self.assertEqual(argv[1:], ["--prepare-proof-material", "--parameter-k", "12"])
            put(self.root / "pp/bls_midnight_2p12", "verified SRS")
            put(stdout_path, {"format": "compact-proof-material/v1", "mode": "prepare",
                              "cache_directory": str(self.root / "pp"), "assets": [],
                              "verification": "all files checked against locked upstream SHA256 tables",
                              "requested_parameter_k": [11] if self.corrupt == "parameters" else [12],
                              "parameters": [{"k": 12, "name": "bls_midnight_2p12"}]})
        elif label == "keygen-apply":
            put(output / "zkir/apply.bzkir", "binary IR")
            for ext in ("prover", "verifier"):
                put(output / f"keys/apply.{ext}", "" if self.corrupt == "empty-key" else ext)
        elif label == "prove-apply":
            self.assertEqual(argv[1:3], ["--did-primitive-reducer", "acc-jubjub-cell"])
            self.assertFalse((output / "keys/raw.prover").exists())
            value = summary()
            if self.corrupt == "binding":
                value["calls"][0]["changed_binding_rejected"] = False
            if self.corrupt == "reorder":
                value["calls"].reverse()
            for name in ("reducer-deployment.bin", "reducer-ledger-after-deploy.bin"):
                put(output / name, "deployed")
            for name in gate.CASES:
                for suffix, artifact in (
                    (".proof", "proof"), (".inputs", "inputs"), ("-transaction.bin", "transaction"),
                    ("-transaction.proof", "tx-proof"), ("-state.bin", "state"),
                    ("-ledger-after.bin", "ledger"),
                    ("-ledger-replay.bin", "mutated" if self.corrupt == "replay-bytes" else "ledger"),
                ):
                    put(output / f"{name}{suffix}", artifact)
            put(output / "proof-result.json", value)
            corrupt_paths = {"node-library-drift": self.node_library, "runtime-drift": self.runtime, "generated-drift": output / "contract/index.js",
                             "material-drift": output / "keys/apply.prover",
                             "capture-drift": directory / "ts-capture.json",
                             "tool-drift": directory / "bin/compactc",
                             "reference-drift": self.root / gate.REFERENCE}
            if self.corrupt in corrupt_paths:
                path = corrupt_paths[self.corrupt]
                path.chmod(0o600)
                put(path, "changed after acceptance")

    def run_gate(self):
        return gate.run_gate(self.root / "run", self.root / "compiler", self.root / "scheme",
                             self.target, self.env, command=self.command)

    def test_complete_qualification_and_exact_selected_proof_cases(self):
        result = self.run_gate()
        self.assertEqual(result["status"], "passed", result.get("error"))
        self.assertEqual(self.calls, ["select-rustfmt", "zkir-version", "freeze-node-runtime", "freeze-ts-runtime",
                                     "compile-jubjub", "compare-formatted-fixture", "model-acc-jubjub-cell", "capture-ts-reference",
                                     "test-jubjub-behavior", "build-proof-runner", "prepare-proof-material",
                                     "keygen-apply", "prove-apply", "recheck-node-runtime"])
        self.assertEqual(result["keygen"], {"operation": "apply", "k": 12, "rows": 2236})
        self.assertEqual(result["oracle_mode"], "retained-original-reference")

    def test_stale_fixture_capabilities_or_oracle_refuses_before_cargo_and_keys(self):
        for fault in ("zkir-version", "fixture", "ir", "capability", "model", "capture", "provenance", "missing-reference"):
            with self.subTest(fault=fault):
                self.corrupt = fault
                result = gate.run_gate(self.root / fault, self.root / "compiler", self.root / "scheme",
                                       self.target, self.env, command=self.command)
                self.assertEqual(result["status"], "failed", result)
                self.assertNotIn("keygen-apply", self.calls)
                self.assertNotIn("test-jubjub-behavior", self.calls)

    def test_keys_require_reviewed_shape_and_nonempty_files(self):
        for fault in ("parameters", "keyshape", "empty-key"):
            with self.subTest(fault=fault):
                self.corrupt = fault
                result = gate.run_gate(self.root / fault, self.root / "compiler", self.root / "scheme",
                                       self.target, self.env, command=self.command)
                self.assertEqual(result["status"], "failed", result)
                self.assertNotIn("prove-apply", self.calls)

    def test_external_proof_server_refuses_before_running_commands(self):
        self.env["MIDNIGHT_PROOF_SERVER"] = "https://example.invalid"
        result = self.run_gate()
        self.assertIn("requires local proving", result["error"])
        self.assertEqual(self.calls, [])

    def test_summary_and_final_identity_drift_refuse(self):
        for fault in ("binding", "reorder", "replay-bytes", "node-library-drift", "runtime-drift", "generated-drift", "material-drift",
                      "capture-drift", "tool-drift", "reference-drift"):
            with self.subTest(fault=fault):
                self.corrupt = fault
                result = gate.run_gate(self.root / fault, self.root / "compiler", self.root / "scheme",
                                       self.target, self.env, command=self.command)
                self.assertEqual(result["status"], "failed", result)
                # Restore owned reference for the next fault.
                put(self.root / gate.REFERENCE, capture())

    def test_generic_validator_refuses_all_missing_binding_apply_replay_proof_controls(self):
        for flag in ("changed_binding_rejected", "applied", "recorded_state_matches", "replay_unchanged",
                     "proof_bytes", "state_bytes", "replay_refusal", "transaction_proof_bytes",
                     "transaction_proof_version", "funded_dust_spends", "separate_verification",
                     "corrupt_proof_rejected"):
            value = summary()
            del value["calls"][3][flag]
            with self.subTest(flag=flag), self.assertRaises(gate.common.GateError):
                gate.primitive.validate_summary(value, gate.ROW, require_v3=True)
        for cases in (summary()["calls"][:-1], summary()["calls"] + summary()["calls"][:1],
                      list(reversed(summary()["calls"]))):
            value = summary()
            value["calls"] = cases
            with self.assertRaises(gate.common.GateError):
                gate.primitive.validate_summary(value, gate.ROW, require_v3=True)
        value = summary()
        value["calls"][0]["operation"] = "raw"
        with self.assertRaises(gate.common.GateError):
            gate.primitive.validate_summary(value, gate.ROW, require_v3=True)

    def test_node_library_identity_requires_existing_non_system_objects(self):
        identity = gate.node_library_identity({"sharedObjects": [str(self.node_library),
                                                                 "/usr/lib/libSystem.B.dylib"]})
        self.assertEqual(identity["file_hashes"], gate.hashes([self.node_library]))
        self.assertEqual(identity["system_libraries"], ["/usr/lib/libSystem.B.dylib"])
        for paths in ([], ["relative.dylib"], [str(self.root / "missing.dylib")], ["/usr/lib/libSystem.B.dylib"]):
            with self.subTest(paths=paths), self.assertRaises(gate.common.GateError):
                gate.node_library_identity({"sharedObjects": paths})

    def test_reference_requires_exact_apply_raw_and_refusal_cases(self):
        for field in ("cases", "rawCases", "refusals"):
            value = copy.deepcopy(capture())
            value[field].pop()
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_capture(value)


if __name__ == "__main__":
    unittest.main()

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
"""Source-only controls for the one-shot native Linux Vector gate."""

import ast
import importlib.util
import json
from pathlib import Path
import re
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ledger9-vector-linux.yml"
VERIFIER = ROOT / ".github/scripts/verify_vector_linux_receipts.py"
spec = importlib.util.spec_from_file_location("verify_vector_linux_receipts", VERIFIER)
assert spec and spec.loader
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class VectorLinuxWorkflowControls(unittest.TestCase):
    def test_exact_sha_trigger_and_fail_closed_graph(self):
        text = WORKFLOW.read_text()
        self.assertIn("'codex/ledger9-vector-linux/*'", text)
        self.assertIn('refs/heads/codex/ledger9-vector-linux/$SOURCE_SHA', text)
        self.assertIn('test "$EXPECTED_SHA" = "$SOURCE_SHA"', text)
        self.assertIn("fetch-depth: 0", text)
        self.assertEqual(text.count("bash .github/scripts/trust_vector_source_key.sh"), 2)
        self.assertIn("git cat-file -e '672bd0f226f47a7d036c5c074ce71fbe1af1925e^{commit}'", text)
        preflight = text.split("  source-preflight:\n", 1)[1].split("  compiler-linux:\n", 1)[0]
        producer = text.split("  compiler-linux:\n", 1)[1].split("  vector-semantics:\n", 1)[0]
        consumer = text.split("  vector-semantics:\n", 1)[1].split("  complete:\n", 1)[0]
        complete = text.split("  complete:\n", 1)[1]
        self.assertIn("test_vector_linux_workflow.py", preflight)
        self.assertIn("needs: source-preflight", producer)
        self.assertIn("uses: ./.github/workflows/compiler-producer.yml", producer)
        self.assertIn("system: x86_64-linux", producer)
        self.assertIn("export-shell: true", producer)
        self.assertIn("needs: [source-preflight, compiler-linux]", consumer)
        for source in ("artifact-id", "shell-artifact-id", "producer-attempt"):
            self.assertIn("needs.compiler-linux.outputs." + source, consumer)
        self.assertIn("compiler_closure.py import", consumer)
        self.assertIn("actions/import-compiler-shell", consumer)
        self.assertIn("toolchain: 1.99.0", consumer)
        self.assertIn("nix build --no-link .#runtime.package", consumer)
        self.assertIn("record_compiler_recipe.py", consumer)
        self.assertIn("verify_vector_linux_receipts.py", consumer)
        self.assertIn('tee "$RUNNER_TEMP/vector-semantic-evidence/$gate.stdout.log"', consumer)
        self.assertIn('--gate-log "$RUNNER_TEMP/vector-semantic-evidence/$gate.stdout.log"', consumer)
        self.assertIn("--all \"$RUNNER_TEMP/vector-semantic-evidence\"", consumer)
        self.assertIn("if: ${{ always() }}", complete)
        for owner in ("PREFLIGHT_RESULT", "PRODUCER_RESULT", "VECTOR_RESULT"):
            self.assertIn('test "$' + owner + '" = success', complete)

    def test_exact_gate_roster_and_shared_cargo_cli(self):
        text = WORKFLOW.read_text()
        match = re.search(r"for gate in ([^;]+); do", text)
        self.assertIsNotNone(match)
        roster = match.group(1).split()
        self.assertEqual(roster, list(verifier.CASES))
        self.assertNotIn("vector_map", roster)
        self.assertIn('--cargo-target "$VECTOR_CARGO_TARGET"', text)
        self.assertIn('if [[ "$gate" != vector_length_refusal ]]', text)
        for case in roster:
            path = ROOT / "tools/compact-rust-backend" / f"check_upstream_ledger9_{case}.py"
            self.assertTrue(path.is_file(), case)
            source = ast.parse(path.read_text())
            main = next(node for node in source.body
                        if isinstance(node, ast.FunctionDef) and node.name == "main")
            self.assertIn("compiler-recipe", ast.unparse(main), case)
            if case != "vector_length_refusal":
                self.assertIn("cargo-target", ast.unparse(main), case)

    def test_pinned_public_signer_is_the_only_admitted_key(self):
        script = (ROOT / ".github/scripts/trust_vector_source_key.sh").read_text()
        self.assertIn("F8851C118C66BF98DB9B1B5DB75A9FB8C6327844", script)
        self.assertIn("test \"$actual\" = \"$fingerprint\"", script)
        self.assertIn("git verify-commit HEAD", script)
        self.assertIn("GNUPGHOME=", script)
        self.assertNotIn("recv-keys", script)

    def test_generated_artifact_mutation_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            generated = {}
            for mode in ("typescript", "both"):
                contract = output / "generated" / mode / "contract"
                contract.mkdir(parents=True)
                (contract / "index.js").write_text("export const value = 1;\n")
                row = {"typescript_sha256": verifier.sha256(contract / "index.js")}
                if mode == "both":
                    (contract / "lib.rs").write_text("pub fn value() {}\n")
                    (contract / "compact-rust-ir.json").write_text("{}\n")
                    row["rust_sha256"] = verifier.sha256(contract / "lib.rs")
                    row["rust_ir_sha256"] = verifier.sha256(contract / "compact-rust-ir.json")
                generated[mode] = row
            receipt = {"generated": generated}
            verifier.validate_generated("vector_witness_return", receipt, output)
            (output / "generated/both/contract/lib.rs").write_text("pub fn changed() {}\n")
            with self.assertRaisesRegex(ValueError, "lib.rs differs"):
                verifier.validate_generated("vector_witness_return", receipt, output)
            receipt["generated"].pop("typescript")
            with self.assertRaisesRegex(ValueError, "inventory differs"):
                verifier.validate_generated("vector_witness_return", receipt, output)

    def test_receipt_source_and_compiler_identity_refusals(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            output = base / "vector_length_refusal"
            output.mkdir()
            compiler = base / "package/bin/compactc"
            compiler.parent.mkdir(parents=True)
            compiler.write_bytes(b"compiler")
            scheme = compiler.with_name("compactc-scheme")
            scheme.write_bytes(b"scheme")
            source_sha = "a" * 40
            tree = "b" * 40
            recipe_path = base / "recipe.json"
            recipe_path.write_text('{"status":"passed","head":"' + source_sha +
                                   '","tree":"' + tree + '"}\n')
            receipt = {
                "head": source_sha, "tree": tree, "source_clean": True,
                "status": "passed", "upstream_commit": verifier.PINNED,
                "platform": "Linux x86_64", "original_files": {"source": "x", "test": "y"},
                "compiler_sha256": verifier.sha256(compiler),
                "scheme_sha256": verifier.sha256(scheme),
                "compiler_recipe": {
                    "path": str(recipe_path), "sha256": verifier.sha256(recipe_path),
                    "head": source_sha, "tree": tree,
                    "store": str(compiler.parent.parent),
                },
                "refusals": {"typescript": "refused", "rust": "refused",
                             "paired": "refused"},
            }
            receipt_path = output / "receipt.json"
            gate_log = base / "vector_length_refusal.stdout.log"
            gate_log.write_text(json.dumps({"status": "passed", "head": source_sha,
                                            "output": str(output)}) + "\n")
            def check():
                receipt_path.write_text(json.dumps(receipt))
                return verifier.validate(
                    "vector_length_refusal", receipt_path, gate_log=gate_log,
                    source_sha=source_sha,
                    tree=tree, compiler=compiler, recipe_path=recipe_path,
                    runtime=base, runtime_snapshot={}, runtime_recipe={})
            self.assertEqual(check()["status"], "passed")
            receipt["head"] = "c" * 40
            with self.assertRaisesRegex(ValueError, "source identity differs"):
                check()
            receipt["head"] = source_sha
            compiler.write_bytes(b"stale compiler")
            with self.assertRaisesRegex(ValueError, "compiler artifact differs"):
                check()
            compiler.write_bytes(b"compiler")
            gate_log.write_text(json.dumps({"status": "failed", "head": source_sha,
                                            "output": str(output)}) + "\n")
            with self.assertRaisesRegex(ValueError, "wrong status"):
                check()


if __name__ == "__main__":
    unittest.main()

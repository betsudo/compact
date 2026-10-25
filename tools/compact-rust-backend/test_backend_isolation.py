#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Proof-profile selection must preserve workload and failure propagation."""
"""Keep the primary compiler and required gates independent of the experiment."""
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BackendIsolation(unittest.TestCase):
    def test_ordinary_gate_imports_without_experiment_modules(self):
        # Exercise the real import graph, including retained application gates.
        code = '''
import importlib.abc, sys
class RefuseExperiment(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.startswith(('forge_', 'qualify_linux_', 'run_native_ci_forge')):
            raise ImportError('deferred experiment import: ' + fullname)
sys.meta_path.insert(0, RefuseExperiment())
import check_compactc_target, did_proof_gate, did_relation_gate
import did_digest_reducer_gate, did_primitive_reducer_gate, acc_jubjub_gate
'''
        subprocess.run([sys.executable, "-B", "-c", code],
                       cwd=Path(__file__).resolve().parent, check=True)

    def test_compiler_and_package_have_no_cache_entrypoint(self):
        for relative in ("compiler/compactc.ss", "compiler/config-params.ss",
                         "compiler/passes.ss", "flake.nix",
                         "tools/compact-rust-backend/src/bin/compactc.rs",
                         "tools/compact-rust-backend/compactc-launcher.sh"):
            source = (ROOT / relative).read_text()
            for token in ("zkir-cache", "COMPACT_ZKIR", "key_cache"):
                self.assertNotIn(token, source, relative)
        self.assertFalse((ROOT / "tools/compact-rust-backend/src/bin/compact-zkir-cache.rs").exists())
        self.assertFalse((ROOT / "tools/compact-rust-backend/src/bin_support/key_cache").exists()
                         and any((ROOT / "tools/compact-rust-backend/src/bin_support/key_cache").iterdir()))

    def test_required_workflows_have_no_experiment_route(self):
        for name in ("build-compiler.yml", "compiler-producer.yml", "rust-backend-checks.yml"):
            source = (ROOT / ".github/workflows" / name).read_text().lower()
            self.assertNotIn("forge", source, name)
            self.assertNotIn("native-snapshot", source, name)
            self.assertNotIn("prepare_native_seed", source, name)
        gate = (ROOT / "tools/compact-rust-backend/check_native_ci.sh").read_text()
        for command in ("--consumer --proof", "check_wallet_handoff.mjs",
                        "check_fixture_outputs.py", "check_rejections.py",
                        "cargo test --workspace", "cargo clippy --workspace"):
            self.assertIn(command, gate)


if __name__ == "__main__":
    unittest.main()

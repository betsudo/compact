#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Exercise the archive consumer's source, projection and Cargo graph checks."""

import json
from pathlib import Path
import tempfile
import tomllib
import unittest

from check_archive_consumer import (
    MACROS, ROOT, RUNTIME, check_graph, contract_record, file_record, project_contract,
)


class ArchiveConsumerTests(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name).resolve()
        self.artifact = self.root / "generated"
        self.destination = self.root / "projected"
        self.contract = self.artifact / "contract"
        self.contract.mkdir(parents=True)
        compiler = self.artifact / "compiler"
        compiler.mkdir()
        self.compatibility = {"runtime_version": "0.2.0"}
        self.source_hashes = {"runtime-rs/src/lib.rs": "a" * 64}
        self.metadata = {
            "schema_version": 1, "distribution": "shared-source", "registry_publication_verified": False,
            "compatibility": self.compatibility, "source_sha256": self.source_hashes,
        }
        self.compatibility_path = compiler / "rust-compatibility.json"
        self.compatibility_path.write_text(json.dumps(self.metadata))
        self.manifest = (
            '[package]\nname = "compact-contract-counter"\n\n[dependencies]\n'
            f'{RUNTIME} = {{ path = "{ROOT / "runtime-rs"}", package = "{RUNTIME}" }}\n'
            f'\n[features]\nledger-transaction = ["{RUNTIME}/ledger-transaction"]\n'
        )
        (self.contract / "Cargo.toml").write_text(self.manifest)
        (self.contract / "lib.rs").write_text("pub fn answer() -> u64 { 42 }\n")
        (compiler / "contract-manifest.json").write_text(json.dumps({
            "contract": contract_record(self.contract),
            "compiler": {"rust-compatibility.json": file_record(self.compatibility_path)},
        }))

    def test_only_projected_runtime_dependency_changes(self):
        projected = project_contract(self.artifact, self.destination, self.compatibility, self.source_hashes)
        expected = tomllib.loads(self.manifest)
        expected["dependencies"][RUNTIME] = {"version": "=0.2.0", "package": RUNTIME}
        self.assertEqual(tomllib.loads((projected / "Cargo.toml").read_text()), expected)
        self.assertEqual((projected / "lib.rs").read_bytes(), (self.contract / "lib.rs").read_bytes())
        self.assertEqual((self.contract / "Cargo.toml").read_text(), self.manifest)

    def test_rejects_generated_rust_that_differs_from_compiler_manifest(self):
        (self.contract / "lib.rs").write_text("pub fn answer() -> u64 { 0 }\n")
        with self.assertRaisesRegex(RuntimeError, "differ from their recorded hashes"):
            project_contract(self.artifact, self.destination, self.compatibility, self.source_hashes)
        self.assertFalse(self.destination.exists())

    def test_rejects_archived_sources_that_differ_from_compiler_selection(self):
        different_archive = {"runtime-rs/src/lib.rs": "b" * 64}
        with self.assertRaisesRegex(RuntimeError, "generated runtime source differs from the release archives"):
            project_contract(self.artifact, self.destination, self.compatibility, different_archive)
        self.assertFalse(self.destination.exists())

    def test_resolved_packages_require_archive_checksums_and_runtime_macro_edge(self):
        source = "registry+https://github.com/rust-lang/crates.io-index"
        consumer = self.root / "consumer"
        consumer.mkdir()
        lock = consumer / "Cargo.lock"
        archives = {name: self.root / "vendor" / f"{name}-0.2.0" for name in (MACROS, RUNTIME)}
        entries = [{"name": name, "version": "0.2.0", "sha256": digit * 64}
                   for name, digit in ((MACROS, "1"), (RUNTIME, "2"))]
        lock_text = "".join(
            f'[[package]]\nname = "{entry["name"]}"\nversion = "0.2.0"\n'
            f'source = "{source}"\nchecksum = "{entry["sha256"]}"\n'
            for entry in entries
        )
        lock.write_text(lock_text)
        macro_edges = [{"pkg": MACROS}]
        metadata = {
            "packages": [
                {"name": name, "id": name, "source": source, "version": "0.2.0",
                 "manifest_path": str(directory / "Cargo.toml")}
                for name, directory in archives.items()
            ] + [{
                "name": "compact-contract-counter", "id": "counter",
                "manifest_path": str(self.destination / "Cargo.toml"),
            }],
            "resolve": {"nodes": [
                {"id": RUNTIME, "deps": macro_edges},
                {"id": "counter", "deps": [{"pkg": RUNTIME}]},
            ]},
        }
        contracts = {"counter": self.destination}
        check_graph(metadata, consumer, contracts, archives, entries)

        lock.write_text(lock_text.replace("1" * 64, "3" * 64))
        with self.assertRaisesRegex(RuntimeError, "archive checksum and source in Cargo.lock"):
            check_graph(metadata, consumer, contracts, archives, entries)
        lock.write_text(lock_text)
        macro_edges.clear()
        with self.assertRaisesRegex(RuntimeError, "archived runtime did not resolve the archived macros"):
            check_graph(metadata, consumer, contracts, archives, entries)


if __name__ == "__main__":
    unittest.main()

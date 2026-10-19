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

"""Finite identity and case-accounting controls; no cryptographic execution."""
import hashlib
from pathlib import Path
import tempfile
import unittest
import qualify_linux_forge as qualification


class LinuxQualificationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        files = {}
        for name in qualification.SOURCE_FILES:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
            files[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        self.manifest = dict(schema=1, source_commit=qualification.PUBLIC, files=files)
        self.rustc = "rustc 1.98.1\nrelease: 1.98.1\nhost: x86_64-unknown-linux-gnu\n"

    def test_exact_fingerprint_order_and_newline_rule(self):
        material = b"".join((self.root / name).read_bytes() for name in qualification.FP_PREFIX)
        material += self.rustc.replace("\n", ";").encode()
        material += (self.root / "src/domain.rs").read_bytes() + b"x86_64linux"
        self.assertEqual(qualification.fingerprint(self.root, self.manifest, self.rustc), hashlib.sha256(material).hexdigest())

    def test_wrong_revision_toolchain_platform_and_source_refuse(self):
        for field, value in [("source_commit", "0" * 40), ("files", {})]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                qualification.fingerprint(self.root, {**self.manifest, field: value}, self.rustc)
        for verbose in [self.rustc.replace("1.98.1", "1.99.0"), self.rustc.replace("x86_64-unknown-linux-gnu", "aarch64-apple-darwin")]:
            with self.assertRaises(ValueError):
                qualification.fingerprint(self.root, self.manifest, verbose)
        (self.root / "src/application.rs").write_bytes(b"changed fingerprint recipe")
        with self.assertRaises(ValueError):
            qualification.fingerprint(self.root, self.manifest, self.rustc)

    def test_actual_derivation_binds_output_platform_rustc_and_source(self):
        toolchain = self.root / 'toolchain'
        forge = self.root / 'output/bin/zkir-forge'
        row = dict(system='x86_64-linux', env=dict(out=str(forge.parent.parent), nativeBuildInputs=str(toolchain), src=str(self.root)))
        for wrapper in [dict(example=row), dict(derivations=dict(example=row), version=4)]:
            self.assertEqual(qualification.check_derivation(wrapper, forge, toolchain, self.manifest), self.root)
        with self.assertRaises(ValueError):
            qualification.check_derivation(dict(example={**row, 'system':'aarch64-darwin'}), forge, toolchain, self.manifest)
        with self.assertRaises(ValueError):
            qualification.check_derivation(dict(example=row), forge, self.root/'other-rustc', self.manifest)
        (self.root/'build.rs').write_bytes(b'changed actual producer source')
        with self.assertRaises(ValueError):
            qualification.check_derivation(dict(example=row), forge, toolchain, self.manifest)

    def lap(self):
        return dict(passed=True, tests_run=2, cases=[dict(circuit=name, acquisition_path="forge-hit",
            artifact_hashes={kind: "a" * 64 for kind in ["source", "binary_ir", "prover", "verifier"]}, proof_and_assertions={"exit_code": 0},
            corrupt_verifier_control={"exit_code": 1, "stderr": "expected header tag 'midnight:verifier-key[v6]:'"}) for name in ["counter", "let_return"]])

    def test_real_case_count_route_and_specific_negative_are_required(self):
        value = self.lap()
        self.assertEqual(set(qualification.checked_lap(value, "forge-hit")), {"counter", "let_return"})
        with self.assertRaises(ValueError):
            qualification.checked_lap(value, "forge-miss")
        for mutate in [lambda d: d.update(passed=False), lambda d: d.update(tests_run=1),
                       lambda d: d['cases'].pop(), lambda d: d['cases'][1].update(circuit="counter"),
                       lambda d: d['cases'][0]['proof_and_assertions'].update(exit_code=1),
                       lambda d: d['cases'][0]['corrupt_verifier_control'].update(exit_code=0),
                       lambda d: d['cases'][0]['corrupt_verifier_control'].update(exit_code=2),
                       lambda d: d['cases'][0]['corrupt_verifier_control'].update(stderr="unrelated failure")]:
            value = self.lap()
            mutate(value)
            with self.subTest(mutate=mutate), self.assertRaises(ValueError):
                qualification.checked_lap(value, "forge-hit")


if __name__ == '__main__':
    unittest.main()

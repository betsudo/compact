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

"""Synthetic source-authority controls; no real upstream checkout is modified."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

import constructor_proof_dependencies as deps


class DependencySourceControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def git_fixture(self):
        root = self.root / "git"
        root.mkdir()
        subprocess.run(["git", "init", "--quiet", str(root)], check=True)
        (root / "lib.rs").write_text("pub fn value() -> u8 { 1 }\n")
        subprocess.run(["git", "-C", str(root), "add", "lib.rs"], check=True)
        tree = deps.git(root, "write-tree").decode().strip()
        # An inert synthetic Git authority object, not a product source commit.
        env = dict(os.environ, GIT_AUTHOR_NAME="Fixture", GIT_AUTHOR_EMAIL="fixture@example.invalid",
                   GIT_COMMITTER_NAME="Fixture", GIT_COMMITTER_EMAIL="fixture@example.invalid")
        revision = subprocess.check_output(["git", "-C", str(root), "-c", "commit.gpgsign=false",
            "commit-tree", tree], input=b"synthetic source authority\n", env=env).decode().strip()
        subprocess.run(["git", "-C", str(root), "update-ref", "HEAD", revision], check=True)
        return root, revision

    def registry_fixture(self):
        root = self.root / "registry/src/fixture-index/example-1.0.0"
        root.mkdir(parents=True)
        files = {"Cargo.toml": b'[package]\nname="example"\nversion="1.0.0"\n',
                 "lib.rs": b"pub fn value() -> u8 { 1 }\n"}
        archive = self.root / "registry/cache/fixture-index/example-1.0.0.crate"
        archive.parent.mkdir(parents=True)
        with tarfile.open(archive, "w:gz") as output:
            for name, data in files.items():
                item = tarfile.TarInfo("example-1.0.0/" + name)
                item.size = len(data)
                output.addfile(item, io.BytesIO(data))
                (root / name).write_bytes(data)
        checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
        return root, archive, checksum, files

    def test_git_actual_blob_and_untracked_build_script(self):
        root, revision = self.git_fixture()
        record = deps.git_source(root, revision)
        deps.unchanged({str(root): record})
        source = root / "lib.rs"
        original = source.read_bytes()
        source.write_bytes(original + b"// altered\n")
        with self.assertRaisesRegex(ValueError, "Git blob bytes differ"):
            deps.git_source(root, revision)
        source.write_bytes(original)
        (root / "build.rs").write_text('fn main() { panic!("untracked"); }')
        with self.assertRaisesRegex(ValueError, "untracked/ignored"):
            deps.git_source(root, revision)

    def test_git_wrong_revision_and_post_admission_drift(self):
        root, revision = self.git_fixture()
        with self.assertRaisesRegex(ValueError, "checkout commit differs"):
            deps.git_source(root, "0" * 40)
        record = deps.git_source(root, revision)
        (root / "lib.rs").write_text("// changed after admission\n")
        with self.assertRaisesRegex(ValueError, "physical dependency bytes changed"):
            deps.unchanged({str(root): record})

    def test_registry_locked_archive_without_optional_checksum_table(self):
        root, archive, checksum, _ = self.registry_fixture()
        package = {"manifest_path": str(root / "Cargo.toml")}
        record = deps.registry_source(package, checksum)
        deps.unchanged({str(root): record})
        archive.write_bytes(archive.read_bytes() + b"altered archive")
        with self.assertRaisesRegex(ValueError, "locked package checksum"):
            deps.registry_source(package, checksum)

    def test_registry_rewritten_table_cannot_rebind_source(self):
        root, _, checksum, files = self.registry_fixture()
        package = {"manifest_path": str(root / "Cargo.toml")}
        table = {"package": checksum, "files": {
            name: hashlib.sha256(data).hexdigest() for name, data in files.items()}}
        path = root / ".cargo-checksum.json"
        path.write_text(json.dumps(table))
        deps.registry_source(package, checksum)
        (root / "lib.rs").write_text("// changed source\n")
        table["files"]["lib.rs"] = hashlib.sha256((root / "lib.rs").read_bytes()).hexdigest()
        path.write_text(json.dumps(table))
        with self.assertRaisesRegex(ValueError, "table differs from locked archive"):
            deps.registry_source(package, checksum)

    def test_registry_extra_build_script_refuses(self):
        root, _, checksum, _ = self.registry_fixture()
        (root / "build.rs").write_text("fn main() {}\n")
        with self.assertRaisesRegex(ValueError, "membership or bytes changed"):
            deps.registry_source({"manifest_path": str(root / "Cargo.toml")}, checksum)


if __name__ == "__main__":
    unittest.main()

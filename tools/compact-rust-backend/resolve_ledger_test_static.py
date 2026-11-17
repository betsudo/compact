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

"""Resolve static fixtures from the exact published Ledger 9 crate.

Cargo.lock, Cargo metadata, the cached crate archive and extracted fixture
bytes must agree. Proving keys still use the official provider and
its pinned identities; this resolver never manufactures or caches proofs.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tomllib

PACKAGE = "midnight-ledger-v9"
VERSION = "1.0.2"
REVISION = "f82ac287442fc15c4eb79f02dcb6510373979eab"
REPOSITORY = "https://github.com/betsudo/midnight-ledger"
SOURCE = f"git+{REPOSITORY}?rev={REVISION}#{REVISION}"
FIXTURE_HASHES = {
    "spend.bzkir": "d211cacb22391b9db60f945787d98c16d344cbb4790246132117084c080399c1",
    "spend.verifier": "d15a1295780549f534287a39c938005e1faa3232be21861573bd0cdb148949cb",
    "spend.zkir": "34a9adee3f20e5042da73432dc081973b53687e7bc30dd9efc046061f3d172cb",
}
PROVER_HASH = "3dff9f54add761350ad02621a425d67fe3d656c8145fef8ef147870489ffdc81"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git_output(crate, *args):
    return subprocess.run(
        ["git", "-C", str(crate), *args], check=True, capture_output=True
    ).stdout


def resolve(metadata, lock):
    packages = [p for p in metadata["packages"] if p["name"] == PACKAGE]
    locked = [p for p in lock["package"] if p["name"] == PACKAGE]
    require(len(packages) == len(locked) == 1, f"expected one {PACKAGE} package")
    package, entry = packages[0], locked[0]
    for item in (package, entry):
        require(item["version"] == VERSION and item["source"] == SOURCE,
                "ledger version/source differs from the audited Git package")
    require("checksum" not in entry, "Git ledger lock entry must not carry a registry checksum")
    manifest = Path(package["manifest_path"])
    require(manifest.is_absolute(), "metadata manifest path must be absolute")
    crate = manifest.parent.resolve(strict=True)
    require(not any(c in str(crate) for c in "\r\n"), "unsafe environment path")
    require(not manifest.is_symlink(), "symlinked ledger manifest path")
    identity = tomllib.loads(manifest.read_text())["package"]
    require(identity["name"] == PACKAGE and identity["version"] == VERSION,
            "resolved manifest identity mismatch")
    repository = Path(git_output(crate, "rev-parse", "--show-toplevel").decode().strip())
    require(crate == repository / "ledger", "ledger package is outside the expected Git source path")
    require(git_output(crate, "rev-parse", "HEAD").decode().strip() == REVISION,
            "ledger checkout revision differs from the locked source")
    static = crate / "static"
    files = ["Cargo.toml", "static/dust/spend.prover.sha256"]
    for name, digest in FIXTURE_HASHES.items():
        require(hashlib.sha256((static / "dust" / name).read_bytes()).hexdigest() == digest,
                f"static fixture digest mismatch: {name}")
        require((static / "dust" / (name + ".sha256")).read_text().split()[0] == digest,
                f"static fixture digest declaration mismatch: {name}")
        files.extend((f"static/dust/{name}", f"static/dust/{name}.sha256"))
    require((static / "dust/spend.prover.sha256").read_text().split()[0] == PROVER_HASH,
            "upstream prover digest declaration mismatch")
    for relative in files:
        path = crate / relative
        require(path.resolve(strict=True) == path and path.is_file(),
                f"ledger source member is not a regular file: {relative}")
        committed = git_output(crate, "show", f"{REVISION}:ledger/{relative}")
        require(committed == path.read_bytes(),
                f"ledger source member differs from the locked Git commit: {relative}")
    return {
        "format": "compact-ledger-test-static/v2",
        "package": PACKAGE, "version": VERSION, "source": SOURCE,
        "source_repository": REPOSITORY, "source_revision": REVISION,
        "static_directory": str(static), "fixture_sha256": FIXTURE_HASHES,
        "upstream_prover_sha256": PROVER_HASH,
        "scope": "External test resolver base; upstream provider supplies and verifies proving material",
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--lock", type=Path, required=True)
    parser.add_argument("--provenance", type=Path, required=True)
    parser.add_argument("--github-env", type=Path)
    args = parser.parse_args(argv)
    try:
        record = resolve(json.loads(args.metadata.read_text()),
                         tomllib.loads(args.lock.read_text()))
        record["cargo_lock_sha256"] = hashlib.sha256(args.lock.read_bytes()).hexdigest()
        # All validation finishes before either output is written.
        args.provenance.write_text(json.dumps(record, indent=2) + "\n")
        if args.github_env:
            with args.github_env.open("a") as output:
                print(f"MIDNIGHT_LEDGER_TEST_STATIC_DIR={record['static_directory']}", file=output)
        print(record["static_directory"])
        return 0
    except (OSError, ValueError, KeyError, TypeError, IndexError,
            subprocess.CalledProcessError) as error:
        print(f"ledger static fixture resolution failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

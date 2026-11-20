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

"""Inventory pinned upstream files and their candidate paths without running tests."""

import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
UPSTREAM = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
SUITES = {
    ".github": "workflow-and-action",
    "compiler": "scheme-compiler",
    "debug-test": "debug-application",
    "editor-support": "editor",
    "examples": "compiler-and-e2e-fixtures",
    "nix": "nix-tooling",
    "runtime": "typescript-runtime",
    "scripts": "repository-tooling",
    "specification": "formal-specification",
    "srcMaps": "source-map-generator",
    "test-center": "generated-contract-execution",
    "test-contracts": "standalone-contract-tests",
    "test-src-maps": "source-map-tests",
    "tests-e2e": "compiler-e2e",
    "third_party": "compiler-dependencies",
    "tools": "cli-and-packaging",
}


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def tree(revision):
    files = {}
    for entry in git("ls-tree", "-rz", "--full-tree", revision).split(b"\0"):
        if entry:
            metadata, path = entry.split(b"\t", 1)
            files[path.decode()] = metadata.decode().split()
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", default="HEAD")
    args = parser.parse_args()
    candidate = git("rev-parse", "--verify", "--end-of-options", args.candidate + "^{commit}").decode().strip()
    upstream_files = tree(UPSTREAM)
    candidate_files = tree(candidate)
    rows = []
    summary = {}
    for path, upstream_entry in sorted(upstream_files.items()):
        suite = SUITES.get(path.split("/", 1)[0], "other-upstream-files")
        current = candidate_files.get(path)
        if current is None:
            comparison = "missing"
        elif current == upstream_entry:
            comparison = "identical"
        else:
            comparison = "changed"
        summary.setdefault(suite, Counter())[comparison] += 1
        rows.append({
            "path": path, "suite": suite, "upstream": upstream_entry,
            "candidate": current, "comparison": comparison,
        })
    print(json.dumps({
        "upstream": UPSTREAM, "candidate": candidate,
        "scope": "Tracked file inventory. Identical bytes do not establish test coverage.",
        "summary": summary, "files": rows,
    }, indent=2))


if __name__ == "__main__":
    main()

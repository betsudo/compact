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

"""Record an actual original-recipe realization of an already available compiler.

Builds, remote builders and substitution downloads are disabled. The result must
be the supplied compiler package. In CI the existing authenticated same-run import
is a separate prerequisite; this recorder does not authenticate GitHub artifacts
or claim another compiler build, proof, or hosted execution.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_state(root, git):
    def text(*arguments):
        return subprocess.check_output([str(git), *arguments], cwd=root, text=True).strip()
    require(not text("status", "--porcelain"), "clean source required")
    return {"head": text("rev-parse", "HEAD"), "tree": text("rev-parse", "HEAD^{tree}")}


def record(compiler, output, *, root=ROOT):
    output = output.absolute()
    require(not output.exists() and not output.is_symlink(), "recipe output must be absent")
    output.mkdir(parents=True)
    report = {"format": "compact-original-existing-compiler-recipe/v1", "status": "incomplete",
              "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "commands": [],
              "scope": __doc__}
    started = time.monotonic()
    try:
        root = root.resolve(strict=True)
        git = Path(shutil.which("git")).resolve(strict=True)
        nix = Path(shutil.which("nix")).resolve(strict=True)
        expected = compiler.resolve(strict=True)
        require(expected.name == "compactc" and expected.is_file(), "expected compiler executable required")
        package = expected.parent.parent
        scheme = (package / "bin/compactc-scheme").resolve(strict=True)
        require(scheme.parent == expected.parent and os.access(expected, os.X_OK)
                and os.access(scheme, os.X_OK), "one executable compiler package with Scheme sibling required")
        identity = source_state(root, git)
        report.update(identity)
        recipe = [root / name for name in ["flake.nix", "flake.lock", "Cargo.lock",
                  "tools/compact-rust-backend/Cargo.lock", "compiler/compiler-version.ss"]]
        pins = {str(p): digest(p) for p in [git, nix, expected, scheme, Path(__file__).resolve(), *recipe]}
        report["inputs_before_use"] = pins

        def unchanged():
            require(source_state(root, git) == identity, "source identity changed during recipe recording")
            require(all(Path(p).is_file() and digest(p) == sha for p, sha in pins.items()),
                    "recipe or tool bytes changed during recording")

        def execute(label, command):
            unchanged()
            out, err = output / (label + ".stdout"), output / (label + ".stderr")
            with out.open("x") as stdout, err.open("x") as stderr:
                result = subprocess.run(command, cwd=root, stdout=stdout, stderr=stderr, check=False)
            report["commands"].append({"label": label, "command": list(map(str, command)),
                "exit": result.returncode, "stdout": str(out), "stdout_sha256": digest(out),
                "stderr": str(err), "stderr_sha256": digest(err)})
            require(result.returncode == 0, label + " failed; retained raw command output")
            unchanged()
            return out.read_text().strip()

        command = [str(nix), "build", "--offline", "--no-link", "--print-out-paths",
                   "--max-jobs", "0", "--builders", "", "--option", "substitute", "false", ".#compactc"]
        realized = execute("existing-recipe", command)
        require(realized == str(package), "original recipe output differs from supplied compiler package")
        require((Path(realized) / "bin/compactc").resolve(strict=True) == expected,
                "original recipe compiler differs from supplied binary")
        version = execute("compiler-version", [str(expected), "--version"])
        require(bool(version) and "\n" not in version, "one compiler version required")
        report.update(store=realized, version=version, command=command,
                      compactc={"path": str(expected), "sha256": pins[str(expected)]})
        report["compactc-scheme"] = {"path": str(scheme), "sha256": pins[str(scheme)]}
        unchanged()
        report["status"] = "passed"
    except Exception as error:
        report.update(status="failed", error=str(error))
        raise
    finally:
        report["seconds"] = time.monotonic() - started
        with (output / "receipt.json").open("x") as stream:
            stream.write(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True,
                        help="already available compiler; CI supplies the authenticated same-run package")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = record(args.compiler, args.output)
    print(json.dumps({"status": result["status"], "head": result["head"],
                      "receipt": str(args.output / "receipt.json")}))


if __name__ == "__main__":
    main()

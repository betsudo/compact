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

"""Run pinned nominal Uint programs with generated TypeScript and Rust."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/primitives/uint/nominal"
PIN = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
SOURCES = {
    "same_range_arithmetic": "uint_nominal_same_range_arithmetic",
    "addition_overflow": "uint_nominal_addition_overflow",
}


def run(*command, **options):
    return subprocess.run(command, cwd=ROOT, check=True, **options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    built = json.loads((runtime / "package.json").read_text())
    source = json.loads((ROOT / "runtime/package.json").read_text())
    if any(built[key] != source[key] for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match this checkout")

    for family, stem in SOURCES.items():
        directory = FAMILY / family
        files = [p for p in directory.iterdir() if p.is_file()]
        if len(files) != 3:
            raise RuntimeError(f"expected exactly three pinned files in {directory}")
        for local in files:
            relative = local.relative_to(ROOT)
            upstream = run("git", "show", f"{PIN}:{relative}", stdout=subprocess.PIPE).stdout
            if local.read_bytes() != upstream:
                raise RuntimeError(f"pinned fixture differs from upstream: {relative}")

    runtime_link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    (output / "package.json").write_text('{"type":"module"}\n')
    dependencies = {}
    expected = {
        "same_range_arithmetic": {"mul_bit", "mul_bit_range", "add_zero", "mul_zero", "sub_bit"},
        "addition_overflow": {"add_byte"},
    }
    for family, stem in SOURCES.items():
        source_file = (FAMILY / family / f"{stem}.compact").relative_to(ROOT)
        default = output / "default" / family
        both = output / "both" / family
        run(compiler, source_file, default)
        run(compiler, "--target", "rust", "--target", "ts", "--feature-zkir-v3",
            "--skip-zk", "--rust-runtime-root", ROOT, source_file, both)
        for generated in (default, both):
            circuits = json.loads((generated / "compiler/contract-info.json").read_text())["circuits"]
            if {row["name"] for row in circuits} != expected[family] or any(
                not row["pure"] or row["proof"] for row in circuits
            ):
                raise RuntimeError(f"{family} must contain only the original pure exports")
            run("node", FAMILY / "check.mjs", generated / "contract/index.js", family)
        dependencies["same" if family == "same_range_arithmetic" else "overflow"] = both / "contract"

    consumer = output / "rust-consumer"
    (consumer / "tests").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "tests/nominal.rs")
    lock = consumer / "Cargo.lock"
    shutil.copyfile(ROOT / "Cargo.lock", lock)
    manifest = consumer / "Cargo.toml"
    lines = ["[workspace]", "[package]", 'name = "upstream-ledger9-nominal-uint-consumer"',
             'version = "0.0.0"', 'edition = "2024"', "[dependencies]"]
    for name, contract in dependencies.items():
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        lines.append(f'{name} = {{ package = {json.dumps(package)}, path = {json.dumps(str(contract))} }}')
    manifest.write_text("\n".join(lines) + "\n")
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(output / "rust-target"))
    cargo = ("cargo", "+1.99.0")
    metadata = json.loads(run(*cargo, "metadata", "--offline", "--format-version", "1",
                              "--manifest-path", manifest, env=env, stdout=subprocess.PIPE).stdout)
    workspace_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    pinned = {(row["name"], row["version"], row["source"]): row.get("checksum")
              for row in workspace_lock if "source" in row}
    local = {manifest, ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml"}
    local.update(contract / "Cargo.toml" for contract in dependencies.values())
    validate_consumer_lock(metadata, lock, local, pinned)
    run(*cargo, "test", "--locked", "--offline", "--manifest-path", manifest, env=env)


if __name__ == "__main__":
    main()

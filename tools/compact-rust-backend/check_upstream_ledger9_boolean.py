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

"""Run the pinned upstream Boolean casts against generated Rust and TypeScript."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/primitives/boolean"
CASTS = ("boolean", "field")


def run(*command, **options):
    return subprocess.run(command, cwd=ROOT, check=True, **options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--ts-runtime", type=Path, required=True, help="Built Compact runtime package directory")
    parser.add_argument("--output", type=Path, required=True, help="Absent directory for generated output and Rust consumer")
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    built_package = json.loads((runtime / "package.json").read_text())
    source_package = json.loads((ROOT / "runtime/package.json").read_text())
    if any(built_package[key] != source_package[key] for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match the checkout")
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    runtime_link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    (output / "package.json").write_text('{"type":"module"}\n')

    dependencies = {}
    for cast in CASTS:
        name = f"boolean_casts_to_{cast}"
        source = (FAMILY / "casts" / f"to_{cast}" / f"{name}.compact").relative_to(ROOT)
        default = output / "default" / name
        both = output / "both" / name
        run(compiler, source, default)
        run(compiler, "--target", "rust", "--target", "ts", "--feature-zkir-v3",
            "--skip-zk", "--rust-runtime-root", ROOT, source, both)
        for generated in (default, both):
            info = json.loads((generated / "compiler/contract-info.json").read_text())
            circuits = info["circuits"]
            if (len(circuits) != 1 or circuits[0]["name"] != name
                    or not circuits[0]["pure"] or circuits[0]["proof"] or circuits[0]["arguments"]):
                raise RuntimeError(f"{name} must remain a parameterless pure circuit with no proof")
            run("node", FAMILY / "check.mjs", generated / "contract/index.js", name)
        dependencies[cast] = both / "contract"

    consumer = output / "rust-consumer"
    (consumer / "tests").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "tests/boolean.rs")
    lock = consumer / "Cargo.lock"
    shutil.copyfile(ROOT / "Cargo.lock", lock)
    manifest = consumer / "Cargo.toml"
    lines = ['[workspace]', '[package]', 'name = "upstream-ledger9-boolean-consumer"',
             'version = "0.0.0"', 'edition = "2024"', '[dependencies]']
    for cast, contract in dependencies.items():
        package = tomllib.loads((contract / "Cargo.toml").read_text())["package"]["name"]
        lines.append(f'{cast} = {{ package = {json.dumps(package)}, path = {json.dumps(str(contract))} }}')
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

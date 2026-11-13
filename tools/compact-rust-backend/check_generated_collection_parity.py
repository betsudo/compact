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

"""Compare fresh dual-target generated collection circuits with retained TS state bytes."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[2]
SOURCES = {
    "set": ROOT / "runtime-rs/tests/fixtures/ledger9-set-transition-v018.compact",
    "map": ROOT / "runtime-rs/tests/fixtures/ledger9-map-transition-v018.compact",
    "list": ROOT / "examples/rust_backend/list_field.compact",
}
PACKAGES = {
    "set": "compact-contract-ledger9-set-transition-v018",
    "map": "compact-contract-ledger9-map-transition-v018",
    "list": "compact-contract-list-field",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], *, env: dict[str, str], cwd: Path = ROOT) -> None:
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}:\n{result.stdout[-4000:]}\n{result.stderr[-4000:]}")


def qualify(output: Path, compiler: Path, scheme: Path) -> dict:
    if output.exists() and any(output.iterdir()):
        raise RuntimeError(f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), COMPACT_REPO_ROOT=str(ROOT))
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    env["RUSTUP_TOOLCHAIN"] = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    artifacts = {}
    for kind, source in SOURCES.items():
        destination = output / kind
        run([
            str(compiler), "--target", "ts", "--target", "rust",
            "--rust-runtime-root", str(ROOT), "--rust-require-recording",
            "--skip-zk", "--compact-path", str(ROOT / "compiler"),
            str(source), str(destination),
        ], env=env)
        contract = destination / "contract"
        artifacts[kind] = {
            "source": str(source.relative_to(ROOT)),
            "source_sha256": sha256(source),
            "oracle_sha256": sha256(ROOT / f"runtime-rs/tests/fixtures/ledger9-{kind}-transition.json"),
            "rust_sha256": sha256(contract / "lib.rs"),
            "typescript_sha256": sha256(contract / "index.js"),
            "ir_sha256": sha256(contract / "compact-rust-ir.json"),
        }
    consumer = output / "consumer"
    (consumer / "src").mkdir(parents=True)
    shutil.copyfile(Path(__file__).parent / "tests/generated_collection_consumer.rs", consumer / "src/main.rs")
    dependencies = "".join(
        f'{PACKAGES[kind]} = {{ path = {json.dumps(str(output / kind / "contract"))}, features = ["ledger-transaction"] }}\n'
        for kind in SOURCES
    )
    (consumer / "Cargo.toml").write_text(
        '[package]\nname = "ledger9-generated-collection-consumer"\nversion = "0.1.0"\nedition = "2024"\n'
        '\n[workspace]\n\n[dependencies]\n' + dependencies
        + f'midnight-compact-runtime = {{ path = {json.dumps(str(ROOT / "runtime-rs"))}, features = ["ledger-transaction"] }}\n'
        + 'midnight-onchain-state = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n'
        + 'midnight-storage = "=3.0.0"\nserde_json = "1"\nhex = "0.4"\n'
    )
    run(["cargo", "generate-lockfile"], env=env, cwd=consumer)
    run(["cargo", "run", "--locked", "--quiet"], env=env, cwd=consumer)
    return {
        "compiler_sha256": sha256(compiler),
        "scheme_frontend_sha256": sha256(scheme),
        "runtime_compatibility_sha256": sha256(ROOT / "runtime-rs/compatibility.json"),
        "rust_toolchain_sha256": sha256(ROOT / "rust-toolchain.toml"),
        "sources_and_artifacts": artifacts,
        "consumer_source_sha256": sha256(consumer / "src/main.rs"),
        "consumer_lock_sha256": sha256(consumer / "Cargo.lock"),
        "result": "Generated Rust initial states and 12 Set/Map/List transitions match retained TypeScript state bytes",
        "platform": "local; native Linux gate remains required",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, default=ROOT / "target/debug/compactc")
    parser.add_argument("--scheme", type=Path, default=os.environ.get("COMPACTC_SCHEME"))
    parser.add_argument("--output-root", type=Path)
    args = parser.parse_args()
    if args.scheme is None or not args.compiler.is_file() or not args.scheme.is_file():
        parser.error("built Rust compactc and Scheme frontend paths are required")
    if args.output_root is None:
        with tempfile.TemporaryDirectory(prefix="ledger9-generated-collections-") as temporary:
            result = qualify(Path(temporary), args.compiler.resolve(), args.scheme.resolve())
    else:
        result = qualify(args.output_root.resolve(), args.compiler.resolve(), args.scheme.resolve())
        (args.output_root / "qualification.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

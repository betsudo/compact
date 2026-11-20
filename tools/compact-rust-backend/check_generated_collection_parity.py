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

from collection_gate_binding import compiler_recipe, git


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


def run(args: list[str], *, env: dict[str, str], cwd: Path = ROOT) -> str:
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}:\n{result.stdout[-4000:]}\n{result.stderr[-4000:]}")
    return result.stdout


def qualify(output: Path, compiler: Path, scheme: Path,
            compiler_receipt: Path) -> dict:
    if git('status', '--porcelain'):
        raise RuntimeError('source checkout must be clean')
    head = git('rev-parse', 'HEAD')
    tree = git('rev-parse', 'HEAD^{tree}')
    compiler_recipe(compiler_receipt, compiler, scheme,
                    source_head=head, current_head=head)
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
    shutil.copyfile(Path(__file__).parent / "fixtures/generated_collection_consumer.rs", consumer / "src/main.rs")
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
    consumer_output = run(["cargo", "run", "--locked", "--quiet"], env=env, cwd=consumer)
    assert "ordinary and recorded/replayed transitions match retained TypeScript oracles" in consumer_output
    if git('status', '--porcelain') or git('rev-parse', 'HEAD') != head:
        raise RuntimeError('source checkout changed during collection qualification')
    return {
        "head": head,
        "tree": tree,
        "compiler_receipt_sha256": sha256(compiler_receipt),
        "compiler_sha256": sha256(compiler),
        "scheme_frontend_sha256": sha256(scheme),
        "runtime_compatibility_sha256": sha256(ROOT / "runtime-rs/compatibility.json"),
        "rust_toolchain_sha256": sha256(ROOT / "rust-toolchain.toml"),
        "sources_and_artifacts": artifacts,
        "consumer_source_sha256": sha256(consumer / "src/main.rs"),
        "consumer_lock_sha256": sha256(consumer / "Cargo.lock"),
        "result": "Generated Rust ordinary and independently replayed Set/Map/List circuits match three initial and twelve transition TypeScript state bytes",
        "consumer_output": consumer_output.strip(),
        "recorded_replay_checks": "state, effects, gas cost for each of 12 mutations",
        "platform": "local; native Linux gate remains required",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler-receipt", type=Path, required=True)
    parser.add_argument("--compiler", type=Path)
    parser.add_argument("--scheme", type=Path)
    parser.add_argument("--output-root", type=Path)
    args = parser.parse_args()
    receipt = args.compiler_receipt.resolve(strict=True)
    build = json.loads(receipt.read_text())
    compiler = Path(build['compactc']['path']).resolve(strict=True)
    scheme = Path(build['compactc-scheme']['path']).resolve(strict=True)
    if ((args.compiler is not None and args.compiler.resolve(strict=True) != compiler) or
            (args.scheme is not None and args.scheme.resolve(strict=True) != scheme)):
        parser.error('compiler or Scheme frontend differs from the source-bound recipe')
    if args.output_root is None:
        with tempfile.TemporaryDirectory(prefix="ledger9-generated-collections-") as temporary:
            result = qualify(Path(temporary), compiler, scheme, receipt)
    else:
        result = qualify(args.output_root.resolve(), compiler, scheme, receipt)
        (args.output_root / "qualification.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

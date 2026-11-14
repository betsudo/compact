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

"""Run the two original mixed foreign-field cast cases through both targets."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/secp256r1/casts/mixed-original"
ARCHIVE = FAMILY.parent
PIN = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
DEPENDENCIES = """hex = "=0.4.3"
serde_json = "1"
midnight-base-crypto = "=1.1.1"
midnight_ledger = { package = "midnight-ledger-v9", version = "=1.0.1", features = ["test-utilities", "proving"] }
midnight-onchain-state = "=4.0.1"
midnight-serialize = "=1.2.1"
midnight-storage = "=3.0.0"
midnight-transient-crypto = "=3.0.1"
midnight-zkir = "=3.2.0"
midnight-zswap = "=9.0.1"
rand = "=0.8.6"
tokio = { version = "=1.50.0", features = ["rt", "time", "net"] }
"""


def run(*command, cwd=ROOT, env=None, stdout=None):
    return subprocess.run(command, cwd=cwd, env=env, stdout=stdout, check=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def values_from_archive(archive):
    binding = re.search(r'\(let \(\[v \(\+ \(expt 2 (\d+)\) (\d+)\)\] \[wide \(\+ \(max-curve25519-scalar\) (\d+)\)\]\)', archive)
    if not binding:
        raise RuntimeError("upstream mixed constant bindings changed")
    constants = (ROOT / "runtime/src/constants.ts").read_text()
    modulus = re.search(r'CURVE25519_SCALAR_MODULUS: bigint = 2n \*\* (\d+)n \+ (\d+)n;', constants)
    if not modulus:
        raise RuntimeError("Curve25519 scalar modulus expression changed")
    v = (1 << int(binding.group(1))) + int(binding.group(2))
    order = (1 << int(modulus.group(1))) + int(modulus.group(2))
    wide = order - 1 + int(binding.group(3))
    return {"v": v, "wide": wide}


def source_lines(lines, values):
    rendered = []
    for line in lines:
        part = line.strip()
        if part.startswith('"') and part.endswith('"'):
            rendered.append(json.loads(part))
        elif part.startswith(",(format "):
            match = re.fullmatch(r',\(format (".*") (v|wide)\)', part)
            if not match:
                raise RuntimeError(f"unexpected upstream interpolation: {part}")
            value = values[match.group(2)]
            rendered.append(json.loads(match.group(1)).replace("~d", str(value)))
    return "\n".join(rendered) + "\n"


def source_from_archive(archive, values):
    return source_lines(archive.split("(stage-javascript", 1)[0].splitlines(), values)


def callback_from_archive(archive, values):
    callback = source_lines(archive.split("(stage-javascript", 1)[1].splitlines(), values)
    # The original test harness compiles this as TypeScript; Node executes JS.
    return callback.replace(": any", "")


def consumer_lock(manifest, contracts, env):
    lock = manifest.parent / "Cargo.lock"
    shutil.copyfile(ROOT / "Cargo.lock", lock)
    metadata = json.loads(run("cargo", "+1.99.0", "metadata", "--offline", "--format-version", "1",
                              "--manifest-path", manifest, env=env, stdout=subprocess.PIPE).stdout)
    workspace_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    pinned = {(row["name"], row["version"], row["source"]): row.get("checksum")
              for row in workspace_lock if "source" in row}
    local = {manifest, ROOT / "runtime-rs/Cargo.toml", ROOT / "runtime-rs-macros/Cargo.toml"}
    local.update(contract / "Cargo.toml" for contract in contracts)
    validate_consumer_lock(metadata, lock, local, pinned)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "scheme-compiler", "ts-runtime", "zkir", "zkir-wasm",
                 "proof-runner", "material", "wrong-verifier", "output", "cargo-target"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    values = vars(args)
    for name in values:
        if name not in ("output", "cargo_target"):
            values[name] = values[name].resolve(strict=True)
    args.cargo_target = args.cargo_target.resolve()
    args.cargo_target.mkdir(parents=True, exist_ok=True)
    source_start = run("git", "status", "--porcelain=v1", stdout=subprocess.PIPE).stdout.decode()
    if source_start:
        parser.error("source checkout must be clean")
    source_head = run("git", "rev-parse", "HEAD", stdout=subprocess.PIPE).stdout.decode().strip()
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    built = json.loads((args.ts_runtime / "package.json").read_text())
    source = json.loads((ROOT / "runtime/package.json").read_text())
    if any(built[key] != source[key] for key in ("name", "version", "dependencies")):
        parser.error("TypeScript runtime package does not match this checkout")
    wasm = json.loads((args.zkir_wasm / "package.json").read_text())
    if wasm["version"] != "3.2.0":
        parser.error("ZKIR wasm package must be 3.2.0")

    upstream = run("git", "show", f"{PIN}:compiler/test.ss", stdout=subprocess.PIPE).stdout.decode()
    license_header = (ARCHIVE / "constant/constant.compact").read_text().split("import CompactStandardLibrary;", 1)[0]
    archives = {}
    bindings = values_from_archive((ARCHIVE / "constant/upstream.ss.txt").read_text())
    for stem in ("constant", "unused"):
        archive = (ARCHIVE / stem / "upstream.ss.txt").read_text()
        if upstream.count(archive) != 1:
            raise RuntimeError(f"{stem} archive differs from the pinned compiler test")
        original = source_from_archive(archive, bindings)
        local = (FAMILY / f"{stem}.compact").read_text()
        if local != license_header + original:
            raise RuntimeError(f"{stem} program differs from evaluated upstream test")
        archives[stem] = archive

    runtime_link = output / "node_modules/@midnight-ntwrk/compact-runtime"
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(args.ts_runtime, target_is_directory=True)
    (output / "package.json").write_text('{"type":"module"}\n')
    compiler_env = os.environ.copy()
    compiler_env["COMPACTC_SCHEME"] = str(args.scheme_compiler)
    generated = {}
    for stem in ("constant", "unused"):
        source_file = FAMILY / f"{stem}.compact"
        default = output / "default" / stem
        both = output / "both" / stem
        run(args.compiler, "--feature-zkir-v3", source_file, default, env=compiler_env)
        run(args.compiler, "--target", "rust", "--target", "ts", "--feature-zkir-v3",
            "--skip-zk", "--rust-runtime-root", ROOT, source_file, both, env=compiler_env)
        generated[stem] = (default, both)
    for index in range(2):
        run("node", FAMILY / "check.mjs",
            generated["constant"][index] / "contract/index.js",
            generated["unused"][index] / "contract/index.js")
        for stem in ("constant", "unused"):
            callback = output / f"upstream-{stem}.js"
            original_callback = callback_from_archive(archives[stem], bindings)
            if original_callback.count("toEqual(") != {"constant": 9, "unused": 5}[stem]:
                raise RuntimeError(f"{stem} upstream ledger assertion count changed")
            callback.write_text(original_callback)
            run("node", FAMILY / "run-upstream-callback.mjs",
                generated[stem][index] / "contract/index.js", callback)

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(args.cargo_target)
    consumer = output / "consumer"
    (consumer / "tests").mkdir(parents=True)
    shutil.copyfile(FAMILY / "consumer.rs", consumer / "tests/mixed.rs")
    constant = generated["constant"][1] / "contract"
    unused = generated["unused"][1] / "contract"
    manifest = consumer / "Cargo.toml"
    manifest.write_text(f'''[workspace]\n[package]\nname = "upstream-ledger9-mixed-foreign-casts"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\nconstant = {{ package = "compact-contract-constant", path = "{constant}" }}\nunused = {{ package = "compact-contract-unused", path = "{unused}" }}\n''')
    consumer_lock(manifest, (constant, unused), env)
    run("cargo", "+1.99.0", "test", "--locked", "--offline", "--manifest-path", manifest, env=env)

    for stem in ("constant", "unused"):
        run(args.zkir, "mock-compile-many", generated[stem][1] / "zkir")
    capture = output / "capture"
    run("node", FAMILY / "capture.mjs", constant / "index.js", args.zkir_wasm, capture)
    proof = output / "proof"
    proof.mkdir()
    raw = generated["constant"][1] / "zkir/castNative.zkir"
    preimage = capture / "castNative.preimage"
    params = args.material / "bls_midnight_2p8"
    for mode, auxiliary in (("prove", params), ("verify", args.wrong_verifier)):
        with (proof / f"{mode}.stdout").open("xb") as stdout:
            run(args.proof_runner, mode, raw, preimage, proof, auxiliary, stdout=stdout)
    if not (proof / "prove.stdout").read_text().startswith("proved k=8,"):
        raise RuntimeError("fresh k=8 proof not reported")
    if "verified in separate process; rejected different verifier, changed binding and corrupt proof" not in (proof / "verify.stdout").read_text():
        raise RuntimeError("independent proof controls not reported")

    funded = output / "funded-consumer"
    funded.mkdir()
    shutil.copyfile(FAMILY / "funded.rs", funded / "main.rs")
    funded_manifest = funded / "Cargo.toml"
    funded_manifest.write_text(f'''[workspace]\n[package]\nname = "ledger9-mixed-foreign-casts-funded"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\nconstant = {{ package = "compact-contract-constant", path = "{constant}", features = ["ledger-transaction"] }}\n{DEPENDENCIES}\n[[bin]]\nname = "ledger9-mixed-foreign-casts-funded"\npath = "main.rs"\n''')
    consumer_lock(funded_manifest, (constant,), env)
    run("cargo", "+1.99.0", "build", "--locked", "--offline", "--manifest-path", funded_manifest, env=env)
    funded_env = env.copy()
    funded_env.update(MIDNIGHT_PP=str(args.material), MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(args.material),
                      RAYON_NUM_THREADS="4")
    binary = args.cargo_target / "debug/ledger9-mixed-foreign-casts-funded"
    with (output / "funded-run.stdout").open("xb") as stdout:
        run(binary, output, output / "funded-result", args.wrong_verifier, env=funded_env, stdout=stdout)
    if json.loads((output / "funded-run.stdout").read_text()).get("status") != "passed":
        raise RuntimeError("funded strict transaction did not pass")
    source_state = run("git", "status", "--porcelain=v1", stdout=subprocess.PIPE).stdout.decode()
    head = run("git", "rev-parse", "HEAD", stdout=subprocess.PIPE).stdout.decode().strip()
    if source_state or head != source_head:
        raise RuntimeError("source changed during qualification")
    tree = run("git", "rev-parse", "HEAD^{tree}", stdout=subprocess.PIPE).stdout.decode().strip()
    compiler_deriver = run("nix-store", "--query", "--deriver", args.compiler.parent.parent,
                           stdout=subprocess.PIPE).stdout.decode().strip()
    derivation = json.loads(run("nix", "derivation", "show", compiler_deriver,
                                stdout=subprocess.PIPE).stdout)["derivations"]
    compiler_source = Path(next(iter(derivation.values()))["env"]["src"])
    emitter_source = compiler_source / "src/stateful/expression.rs"
    if sha(emitter_source) != sha(ROOT / "tools/compact-rust-backend/src/stateful/expression.rs"):
        raise RuntimeError("Nix compiler emitter source differs from checkout")
    runtime_files = sorted((args.ts_runtime / "dist").rglob("*.js"))
    runtime_hash = hashlib.sha256()
    for path in runtime_files:
        relative = str(path.relative_to(args.ts_runtime)).encode()
        data = path.read_bytes()
        runtime_hash.update(len(relative).to_bytes(8, "big"))
        runtime_hash.update(relative)
        runtime_hash.update(len(data).to_bytes(8, "big"))
        runtime_hash.update(data)
    toolchain = {
        "platform": f"{platform.system()} {platform.machine()}",
        "source_head": head,
        "source_tree": tree,
        "source_clean": not source_state,
        "source_status": source_state.splitlines(),
        "compiler_deriver": compiler_deriver,
        "compiler_source": {"path": str(compiler_source), "emitter_sha256": sha(emitter_source)},
        "proof_runner_source": {name: {"path": str(path), "sha256": sha(path)}
                                for name, path in {
                                    "main": args.proof_runner.parent / "consumer/src/main.rs",
                                    "manifest": args.proof_runner.parent / "consumer/Cargo.toml",
                                    "lock": args.proof_runner.parent / "consumer/Cargo.lock",
                                }.items()},
        "upstream_pin": PIN,
        "tools": {name: {"path": str(path), "sha256": sha(path)} for name, path in {
            "compiler": args.compiler,
            "scheme_compiler": args.scheme_compiler,
            "zkir": args.zkir,
            "zkir_wasm_node": args.zkir_wasm / wasm["exports"]["node"],
            "zkir_wasm_binary": args.zkir_wasm / "midnight_zkir_wasm_bg.wasm",
            "proof_runner": args.proof_runner,
            "params_k8": params,
            "wrong_verifier": args.wrong_verifier,
            "typescript_runtime_package_json": args.ts_runtime / "package.json",
        }.items()},
        "typescript_runtime_dist_js": {"files": len(runtime_files), "sha256": runtime_hash.hexdigest()},
        "checks": {"original_callbacks_per_target": 2, "targets": 2,
                   "rust_consumer_tests": 2, "proof_k": 8, "funded_default_strict": True,
                   "replay_refusal": "IntentAlreadyExists"},
    }
    (output / "toolchain.json").write_text(json.dumps(toolchain, indent=2) + "\n")
    print(f"mixed original: 2 TypeScript targets, 2 Rust tests, fresh proof and funded replay passed; {output}")


if __name__ == "__main__":
    main()

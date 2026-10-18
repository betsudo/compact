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

"""Five fresh functional proof cases using the shared local lifecycle."""
import json
from pathlib import Path
import re
import shutil
import tomllib
import functional_proof_lifecycle as lifecycle
ROOT, RUNNER, forge = lifecycle.ROOT, lifecycle.RUNNER, lifecycle.forge
STAGED = Path(__file__).resolve().parent / "tests/functional_effects/proof_overlays/core"
PACKAGE = "compact-functional-core-proof-smoke"
SPECS = {
    "merkle": {"source": "examples/rust_backend/effect_plan_merkle.compact", "circuits": ["run"],
        "commands": [["--effect-plan-merkle"]], "markers": ["functional Merkle branch false proved", "functional Merkle branch true proved"],
        "alias": "compact-rust-effect-plan-merkle-source-fixture", "retained": "effect-plan-merkle-schema20-ir.json", "module": "effect_plan_merkle.rs"},
    "crypto": {"source": "tools/compact-rust-backend/tests/functional_effects/crypto_operations.compact", "circuits": ["hashing", "jubjub"],
        "commands": [["--functional-crypto", "hashing"], ["--functional-crypto", "jubjub"]],
        "markers": ["functional crypto hashing proved, verified and ledger-applied", "functional crypto jubjub proved, verified and ledger-applied"]},
    "exact": {"source": "examples/rust_backend/effect_plan_exact_hash_input.compact", "circuits": ["witnessed"],
        "commands": [["--exact-hash-input"]], "markers": ["functional exact hash proved and applied"],
        "alias": "compact-rust-effect-plan-exact-hash-input-source-fixture", "retained": "effect-plan-exact-hash-input-schema20-ir.json", "module": "exact_hash_input.rs"},
}


def overlay(runner, generated, spec):
    shutil.copytree(RUNNER, runner)
    manifest = runner / "Cargo.toml"
    text = manifest.read_text()
    forge.require(tomllib.loads(text)["package"]["name"] == "compact-rust-proof-smoke", "canonical runner identity changed")
    text, count = re.subn(r'^name = "compact-rust-proof-smoke"$', f'name = "{PACKAGE}"', text, flags=re.MULTILINE)
    forge.require(count == 1, "private package rename not unique")
    def absolute(match):
        path = Path(json.loads(match[1]))
        return "path = " + json.dumps(str(path if path.is_absolute() else (RUNNER / path).resolve(strict=True)))
    text = re.sub(r'path = ("[^"\n]+")', absolute, text)
    generated_name = tomllib.loads((generated / "Cargo.toml").read_text())["package"]["name"]
    expected_generated = {"merkle":"compact-contract-effect-plan-merkle", "crypto":"compact-contract-crypto-operations", "exact":"compact-contract-effect-plan-exact-hash-input"}[spec["name"]]
    forge.require(generated_name == expected_generated, "fresh generated package identity differs")
    allowed = {str((generated / "Cargo.toml").resolve()), str(manifest.resolve())}
    if "alias" in spec:
        # Preserve the frozen source authority, independently of the fresh IR.
        wrapper = runner.parent / (spec["name"] + "-wrapper")
        wrapper.mkdir()
        frozen = ROOT / "tools/compact-rust-backend/tests" / spec["retained"]
        (wrapper / "lib.rs").write_text('pub use fresh::*;\npub const RETAINED_IR: &str = include_str!(' + json.dumps(str(frozen)) + ');\n')
        (wrapper / "Cargo.toml").write_text('[package]\nname=' + json.dumps(spec["alias"]) + '\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[features]\nledger-transaction=["fresh/ledger-transaction"]\n[dependencies]\nfresh={path=' + json.dumps(str(generated)) + ',package=' + json.dumps(generated_name) + '}\n')
        allowed.add(str((wrapper / "Cargo.toml").resolve()))
        pattern = re.compile(r'^' + re.escape(spec["alias"]) + r' = \{[^\n]+\}$', re.MULTILINE)
        text, count = pattern.subn(lambda _: spec["alias"] + ' = { path = ' + json.dumps(str(wrapper)) + ', features = ["ledger-transaction"] }', text)
        forge.require(count == 1, "selected source alias not unique")
        receipts = json.loads((STAGED / "patch-premises.json").read_text())
        relative = "src/" + spec["module"]
        forge.require(forge.digest(RUNNER / relative) == receipts[relative]["canonical_sha256"], "canonical patch source changed")
        forge.require(forge.digest(STAGED / spec["module"]) == receipts[relative]["overlay_sha256"], "approved overlay changed")
        shutil.copyfile(STAGED / spec["module"], runner / relative)
    else:
        main = runner / "src/main.rs"
        main.write_text(main.read_text() + "\nmod functional_crypto;\n")
        shutil.copyfile(STAGED / "functional_crypto.rs", runner / "src/functional_crypto.rs")
        dispatch = runner / "src/direct_dispatch.rs"
        anchor = '    match (selector, asset) {'
        original = dispatch.read_text()
        forge.require(original.count(anchor) == 1, "proof selector insertion anchor changed")
        dispatch.write_text(original.replace(anchor, anchor + '\n        ("--functional-crypto", _) => Some(super::functional_crypto::arguments(arguments)),'))
        forge.require(text.count("[dependencies]") == 1, "dependency anchor changed")
        text = text.replace("[dependencies]", '[dependencies]\nfunctional-crypto-contract = { path = ' + json.dumps(str(generated)) + ', package = ' + json.dumps(generated_name) + ', features = ["ledger-transaction"] }')
    profile = tomllib.loads((ROOT / "Cargo.toml").read_text())["profile"]["proof-checks"]
    forge.require(profile == {"inherits":"dev", "debug":0, "debug-assertions":True, "overflow-checks":True, "package":{"*":{"opt-level":3}}}, "proof profile policy changed")
    manifest.write_text(text + '\n[workspace]\n[profile.proof-checks]\ninherits="dev"\ndebug=0\ndebug-assertions=true\noverflow-checks=true\n[profile.proof-checks.package."*"]\nopt-level=3\n')
    shutil.copyfile(ROOT / "Cargo.lock", runner / "Cargo.lock")
    return generated_name, allowed


lifecycle.SPECS = SPECS
lifecycle.PACKAGE = PACKAGE
lifecycle.STAGED = STAGED
lifecycle.DRIVER = Path(__file__).resolve()
lifecycle.overlay = overlay
lifecycle.EXTRA_PREMISES = [Path(__file__).resolve(), Path(lifecycle.__file__).resolve(),
    ROOT / "tools/compact-rust-backend/tests/effect-plan-merkle-schema20-ir.json",
    ROOT / "tools/compact-rust-backend/tests/effect-plan-exact-hash-input-schema20-ir.json"]

if __name__ == "__main__":
    raise SystemExit(lifecycle.main())

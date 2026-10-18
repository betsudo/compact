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

"""Finite Kernel/Zswap/qualified-storage proof qualification.

Use --compile-only before real cold/warm/uncached proof qualification.
"""
import json
from pathlib import Path
import re
import shutil
import sys
import tomllib

import functional_proof_lifecycle as lifecycle

STAGED = Path(__file__).resolve().parent / "tests/functional_effects/proof_overlays/domain"
ROOT, RUNNER, forge = lifecycle.ROOT, lifecycle.RUNNER, lifecycle.forge

def binding(root, alias, source):
    return (root, "compact-rust-"+alias+"-fixture", "compact-contract-"+source.replace("_", "-"), source.replace("_oracle", "").replace("_", "-")+"-schema20-ir.json")

SPECS = {
    "kernel": {
        "source":"examples/rust_backend/kernel_shielded_effects_oracle.compact",
        "circuits":["mint","nullifier","spend","claim_receive","batch","selected","witness_order"],
        "commands":[["--kernel-shielded-effects"]],
        "negative_artifacts":[("primary","mint")],
        "markers":["mint selected=false call proof verified; no transaction acceptance claim",
            *[name+" call proof verified; unmatched-offer transaction rejected exactly:" for name in ["nullifier","spend","claim_receive","batch","selected"]],
            "witness_order selected=false call proof verified; no transaction acceptance claim",
            "nonzero Kernel mint42 + matching minted output + Night-backed Dust passed default strict validation and ledger application"],
        "refusal_markers":["selected(false) records/replays zero effects and retains exact EmptyTranscript preparation refusal"],
        "bindings":[binding("primary","kernel-shielded-effects-oracle","kernel_shielded_effects_oracle")],
        "modules":["src/kernel_shielded_effects.rs"],
    },
    "composite": {
        "source":"examples/rust_backend/stateful_struct_oracle.compact",
        "source_inputs":[("examples/rust_backend/stateful_struct_oracle.compact","primary"),("examples/rust_backend/composite_zswap_transfer_oracle.compact","companion")],
        "circuits":["planned"],"artifacts":[("primary","planned"),("companion","transfer")],
        "commands":[["--composite-zswap"]],"command_roots":["primary","companion"],
        "markers":["planned: exact offer + separate Dust, default strictness and ledger apply passed", "transfer: exact offer + separate Dust, default strictness and ledger apply passed"],
        "refusal_markers":["planned negatives: exact -42 deficit, extra wallet input mismatch, missing receive claim under default strictness","composite transfer42: exact spent-nullifier replay rejection"],
        "bindings":[binding("primary","stateful-struct-oracle","stateful_struct_oracle"),binding("companion","composite-zswap-transfer-oracle","composite_zswap_transfer_oracle")],
        "modules":["src/composite_zswap.rs"],
    },
    "cell": {
        "source":"examples/rust_backend/qualified_coin_cell_oracle.compact","circuits":["write_coin"],
        "commands":[["--qualified-coin-cell"]],
        "markers":["qualified coin Cell writeCoin output-backed proof verified and ledger-8 applied with balancing disabled", "funded qualified coin Cell writeCoin passed default strict validation and ledger-8 application"],
        "refusal_markers":["unfunded qualified coin offer rejected by default strictness:"],
        "bindings":[binding("primary","qualified-coin-cell-oracle","qualified_coin_cell_oracle")],
        "modules":["src/qualified_coin_cell.rs"],
    },
    "set": {
        "source":"examples/rust_backend/qualified_coin_set_oracle.compact","circuits":["insert_coin"],
        "commands":[["--qualified-coin-set"]],
        "markers":["qualified coin Set insertCoin output-backed proof verified and ledger-8 applied with balancing disabled", "funded qualified coin Set insertCoin passed default strict validation and ledger-8 application"],
        "refusal_markers":["unfunded qualified coin offer rejected by default strictness:"],
        "bindings":[binding("primary","qualified-coin-set-oracle","qualified_coin_set_oracle")],
        "modules":["src/qualified_coin_set.rs","src/qualified_coin_set/scenario.rs"],
    },
}
for spec in SPECS.values():
    spec["owner_aliases"]=[row[1] for row in spec["bindings"]]


def overlay(runner, primary_contract, spec):
    """Only replace source-specific producers; retain original authority helpers."""
    shutil.copytree(RUNNER, runner)
    receipts=json.loads((STAGED/"patch-premises.json").read_text())
    for relative in spec["modules"]:
        row=receipts[relative]
        forge.require(forge.digest(RUNNER/relative)==row["canonical_sha256"],"canonical domain module changed")
        staged=STAGED/row["staged_file"]
        forge.require(forge.digest(staged)==row["overlay_sha256"],"staged domain overlay changed")
        destination=runner/row.get("private_overlay_destination",relative)
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(staged,destination)
    manifest=runner/"Cargo.toml"
    text=manifest.read_text()
    forge.require(tomllib.loads(text)["package"]["name"]=="compact-rust-proof-smoke","canonical runner identity changed")
    text,count=re.subn(r'^name = "compact-rust-proof-smoke"$', 'name = "'+lifecycle.PACKAGE+'"',text,flags=re.MULTILINE)
    forge.require(count==1,"private package rename not unique")
    def absolute(match):
        path=Path(json.loads(match[1]))
        return "path = "+json.dumps(str(path if path.is_absolute() else (RUNNER/path).resolve(strict=True)))
    text=re.sub(r'path = ("[^"\n]+")',absolute,text)
    allowed={str(manifest.resolve())}
    owners=set()
    directory=primary_contract.parent.parent
    for root_key,alias,expected_generated,retained in spec["bindings"]:
        generated=primary_contract if root_key=="primary" else directory/root_key/"contract"
        actual=tomllib.loads((generated/"Cargo.toml").read_text())["package"]["name"]
        forge.require(actual==expected_generated,"generated package identity changed")
        frozen=ROOT/"tools/compact-rust-backend/tests"/retained
        forge.require(frozen.is_file(),"independent retained IR missing")
        wrapper=runner.parent/(root_key+"-"+spec["name"]+"-wrapper")
        wrapper.mkdir()
        (wrapper/"lib.rs").write_text('pub use fresh::*;\npub const RETAINED_IR: &str=include_str!('+json.dumps(str(frozen))+');\n')
        (wrapper/"Cargo.toml").write_text('[package]\nname='+json.dumps(alias)+'\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[features]\nledger-transaction=["fresh/ledger-transaction"]\n[dependencies]\nfresh={path='+json.dumps(str(generated))+',package='+json.dumps(actual)+'}\n')
        pattern=re.compile(r'^'+re.escape(alias)+r' = \{[^\n]+\}$',re.MULTILINE)
        text,count=pattern.subn(lambda _:alias+' = { path = '+json.dumps(str(wrapper))+', features = ["ledger-transaction"] }',text)
        forge.require(count==1,"source fixture alias not unique")
        allowed.update([str((generated/"Cargo.toml").resolve()),str((wrapper/"Cargo.toml").resolve())])
        owners.update([alias,actual])
    profile=tomllib.loads((ROOT/"Cargo.toml").read_text())["profile"]["proof-checks"]
    forge.require(profile=={"inherits":"dev","debug":0,"debug-assertions":True,"overflow-checks":True,"package":{"*":{"opt-level":3}}},"proof profile policy changed")
    manifest.write_text(text+'\n[workspace]\n[profile.proof-checks]\ninherits="dev"\ndebug=0\ndebug-assertions=true\noverflow-checks=true\n[profile.proof-checks.package."*"]\nopt-level=3\n')
    shutil.copyfile(ROOT/"Cargo.lock",runner/"Cargo.lock")
    return owners,allowed


lifecycle.SPECS=SPECS
lifecycle.PACKAGE="compact-functional-domain-proof-smoke"
lifecycle.STAGED=STAGED
lifecycle.DRIVER=Path(__file__).resolve()
lifecycle.overlay=overlay
lifecycle.EXTRA_PREMISES=[Path(__file__).resolve(),Path(lifecycle.__file__).resolve(),
    *[ROOT/"tools/compact-rust-backend/tests"/row[3] for spec in SPECS.values() for row in spec["bindings"]]]

if __name__=="__main__":
    raise SystemExit(lifecycle.main())

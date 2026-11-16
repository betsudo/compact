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

"""Prove both pinned disclose exports with independent verification and funded replay refusal."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_upstream_ledger9_keyword_constructor_parameters_proof import (
    copy_new, dependency_refusals, files, physical_sha256, run,
    tracked_source_hashes,
)
from check_upstream_ledger9_field_casts import require, sha256, source_state
from check_upstream_ledger9_vector_casts import compiler_recipe_identity
from did_proof_gate import hashes, unchanged
import proof_runner_input_binding as runner_inputs
import resolve_ledger_test_static as ledger_static

ROOT = Path(__file__).resolve().parents[2]
PROOF_BODY = ROOT / 'tools/compact-rust-proof-smoke/src/keyword_disclose_proof.rs'
CASES = ('branch_conditions', 'branch_conditions_in_constructor')
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
WRONG = {
    'branch_conditions': ('if_empty_else_branch',
                          Path('test-contracts/keywords/if/empty_else_branch/if_empty_else_branch.compact')),
    'branch_conditions_in_constructor': ('type_new_type_of_adt',
                                         Path('test-contracts/keywords/type/new_type_of_adt/type_new_type_of_adt.compact')),
}


def qualified_case(case: str, gate_root: Path, gate: dict, compiler: Path,
                   scheme: Path, zkir: Path, cargo_target: Path,
                   parameter_cache: Path, source_before: dict[str, str],
                   source: dict[str, str], output: Path) -> dict:
    circuit = 'disclose_' + case
    wrong, wrong_source = WRONG[case]
    selected = gate_root / case / 'paired/product'
    selected_files = gate['cases'][case]['paired']['generated_files']
    require(files(selected) == selected_files,
            'exact-head disclose SDK inventory changed')
    require(selected_files['contract/index.js'] ==
            gate['cases'][case]['typescript']['generated_files']['contract/index.js'] and
            selected_files[f'zkir/{circuit}.zkir'] ==
            gate['cases'][case]['typescript']['generated_files'][f'zkir/{circuit}.zkir'],
            'selected TypeScript/Rust/ZKIR output differs')
    output.mkdir()
    require(sha256(ROOT / wrong_source) == hashlib.sha256(subprocess.check_output(
            ['git', 'show', f'{PINNED}:{wrong_source}'], cwd=ROOT)).hexdigest(),
            'pinned independent wrong-verifier source changed')
    wrong_generated = output / 'wrong-generated'
    wrong_env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    run([str(compiler), '--target', 'ts', '--feature-zkir-v3', '--skip-zk',
         '--compact-path', str(ROOT / 'compiler'), str(ROOT / wrong_source),
         str(wrong_generated)], output / 'wrong-compile.log', wrong_env)
    wrong_files = files(wrong_generated)
    proof = output / 'proof'
    copy_new(selected / 'contract/compact-rust-ir.json',
             proof / f'{circuit}/contract/compact-rust-ir.json')
    copy_new(selected / f'zkir/{circuit}.zkir', proof / f'zkir/{circuit}.zkir')
    copy_new(wrong_generated / f'zkir/{wrong}.zkir', proof / f'zkir/{wrong}.zkir')
    require(sha256(proof / f'zkir/{circuit}.zkir') !=
            sha256(proof / f'zkir/{wrong}.zkir'),
            'wrong disclose verifier source equals selected circuit')
    (proof / 'keys').mkdir()
    input_hashes = files(proof)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', RUSTUP_TOOLCHAIN='1.99.0',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
               COMPACTC_SCHEME=str(scheme),
               COMPACT_KEYWORD_DISCLOSE_GENERATED=str(selected),
               COMPACT_KEYWORD_DISCLOSE_CASE=case,
               COMPACT_KEYWORD_DISCLOSE_CIRCUIT=circuit,
               COMPACT_KEYWORD_DISCLOSE_WRONG=wrong,
               CARGO_TARGET_DIR=str(cargo_target), MIDNIGHT_PP=str(parameter_cache))
    env.pop('MIDNIGHT_PROOF_SERVER', None)
    metadata = run(['cargo', '+1.99.0', 'metadata', '--locked', '--offline',
                    '--format-version', '1'], output / 'metadata.log', env)
    static = ledger_static.resolve(json.loads(metadata.stdout),
                                   tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = static['static_directory']
    binding_dir = output / 'binding'
    binding_dir.mkdir()
    source_hashes_path = binding_dir / 'source-hashes.json'
    source_hashes_path.write_text(json.dumps(source_before, indent=2, sort_keys=True) + '\n')
    binding = {
        'source_hashes': {str((ROOT / name).resolve(strict=True)): digest
                          for name, digest in source_before.items()},
        'generated_input_hashes': hashes(path for path in selected.rglob('*')
                                         if path.is_file()),
    }

    def execute(label: str, command: list[str], log: Path):
        for field in ('source_hashes', 'generated_input_hashes',
                      'dependency_source_hashes', 'generated_build_inputs_before_compile'):
            if field in binding:
                unchanged(binding[field])
        return run(command, log, env)

    runner_inputs.prepare_generated_inputs(ROOT, binding_dir, binding, execute,
                                           'keyword-disclose-proof', sha256)
    build = execute('build-proof-runner',
                    ['cargo', '+1.99.0', 'build', '--locked', '--offline', '--profile',
                     'proof-checks', '-p', 'compact-rust-proof-smoke', '--features',
                     'keyword-disclose-proof', '--message-format=json'],
                    output / 'build.log')
    rows = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
    executables = {row['executable'] for row in rows
                   if row.get('reason') == 'compiler-artifact' and
                   row['target']['name'] == 'compact-rust-proof-smoke' and row.get('executable')}
    require(len(executables) == 1, 'ambiguous disclose proof runner')
    built = Path(executables.pop())
    runner = output / 'proof-runner'
    copy_new(built, runner)
    runner.chmod(0o755)
    depfile = built.with_suffix('.d')
    tracked = runner_inputs.depfile_prerequisites(depfile.read_text(), ROOT)
    for expected in (selected / 'contract/lib.rs',
                     selected / 'contract/compact-rust-ir.json',
                     selected / f'zkir/{circuit}.zkir', PROOF_BODY):
        require(str(expected.resolve(strict=True)) in tracked,
                f'runner depfile omitted {expected}')
    inventories = {field: binding[field] for field in
                   ('source_hashes', 'generated_input_hashes',
                    'dependency_source_hashes', 'generated_build_inputs_before_compile')}
    physical, origins = runner_inputs.qualify_dependency_inputs(tracked, inventories,
                                                                 physical_sha256)
    refusals = dependency_refusals(tracked, inventories, binding_dir)
    inventory_path = binding_dir / 'inputs.json'
    inventory_path.write_text(json.dumps({**binding,
                                          'physical_inputs_before_build': physical,
                                          'physical_input_origins': origins},
                                         indent=2, sort_keys=True) + '\n')
    retained_depfile = output / 'proof-runner.d'
    copy_new(depfile, retained_depfile)
    runner_sha = sha256(runner)
    depfile_sha = sha256(retained_depfile)
    inventory_sha = sha256(inventory_path)
    require(files(proof) == input_hashes and files(selected) == selected_files and
            files(wrong_generated) == wrong_files,
            'runner build changed proof inputs')
    for field, fragment in ((f'zkir/{circuit}.zkir', 'raw ZKIR bytes differ'),
                            (f'{circuit}/contract/compact-rust-ir.json',
                             'complete Rust IR bytes differ')):
        bad = output / ('changed-zkir' if field.startswith('zkir') else 'changed-ir')
        shutil.copytree(proof, bad)
        changed = bad / field
        changed.write_bytes(changed.read_bytes() + b' ')
        result = subprocess.run([str(runner), '--keyword-disclose-proof', str(bad)],
                                cwd=ROOT, env=env, capture_output=True, text=True)
        (output / f'{bad.name}.log').write_text(result.stdout + result.stderr)
        require(result.returncode != 0 and fragment in result.stderr and
                not list(bad.rglob('*.proof')),
                f'{field} changed without pre-key refusal')
    material = run([str(runner), '--prepare-proof-material', '--verify-only',
                    '--parameter-k', '6', '--parameter-k', '7'],
                   output / 'material.json', env)
    verified = json.loads(material.stdout)
    require(verified['mode'] == 'verify-only' and
            verified['verification'] == 'all files checked against locked upstream SHA256 tables',
            'official upstream proving material unverified')
    material_hashes = {row['name']: sha256(Path(verified['cache_directory']) / row['name'])
                       for row in verified['assets'] + verified['parameters']}
    keys = {}
    for name in (circuit, wrong):
        generated = run([str(zkir), 'compile', str(proof / f'zkir/{name}.zkir'),
                         str(proof / f'keys/{name}.prover'),
                         str(proof / f'keys/{name}.verifier')],
                        output / f'{name}-keygen.log', env)
        match = re.search(r'k=(\d+), rows=(\d+)', generated.stdout + generated.stderr)
        require(match is not None and int(match[1]) in (6, 7),
                f'{name} unqualified key model')
        keys[name] = {'k': int(match[1]), 'rows': int(match[2]),
                      'prover_sha256': sha256(proof / f'keys/{name}.prover'),
                      'verifier_sha256': sha256(proof / f'keys/{name}.verifier'),
                      'binary_zkir_sha256': sha256(proof / f'zkir/{name}.bzkir')}
    require(keys[circuit]['k'] == keys[wrong]['k'] and
            keys[circuit]['verifier_sha256'] != keys[wrong]['verifier_sha256'],
            'same-k independent wrong verifier is absent')
    run([str(runner), '--keyword-disclose-proof', str(proof)],
        output / 'proof-run.log', env)
    summary = json.loads((proof / f'{circuit}-proof-result.json').read_text())
    require(summary['status'] == 'passed' and
            summary['selected_source_export_proved'] == circuit and
            summary['result'] == summary['ledger_x'] == 3 and
            summary['positive_fee_dust_spends'] == 1 and
            summary['strict_application'] == 'default' and
            summary['replay_refusal'] == 'IntentAlreadyExists' and
            all(summary[key] is True for key in
                ('standalone_verified_separately', 'wrong_verifier_rejected_at_preparation',
                 'wrong_verifier_proof_rejected', 'corrupt_proof_rejected',
                 'changed_binding_rejected', 'replay_unchanged')),
            'disclose proof or funded strict replay changed')
    for inventory in inventories.values():
        unchanged(inventory)
    unchanged(physical)
    require(source_state() == source and tracked_source_hashes() == source_before and
            files(selected) == selected_files and files(wrong_generated) == wrong_files and
            sha256(PROOF_BODY) == source_before[str(PROOF_BODY.relative_to(ROOT))] and
            sha256(runner) == runner_sha and sha256(retained_depfile) == depfile_sha and
            sha256(inventory_path) == inventory_sha and
            all(sha256(Path(verified['cache_directory']) / name) == digest
                for name, digest in material_hashes.items()) and
            all(sha256(proof / name) == digest for name, digest in input_hashes.items()),
            'proof source, generated artifact or proving material drifted')
    return {'case':case, 'selected_source_export':circuit,
            'cargo_target':str(cargo_target),
            'generated_input_sha256':selected_files,
            'wrong_generated_sha256':wrong_files,
            'runner_sha256':runner_sha, 'runner_depfile_sha256':depfile_sha,
            'runner_inventory_sha256':inventory_sha,
            'runner_physical_input_count':len(physical),
            'runner_physical_input_origins':origins,
            'dependency_refusals':refusals, 'zkir_sha256':sha256(zkir),
            'ledger_static':static, 'material':verified,
            'material_hashes':material_hashes, 'keygen':keys,
            'proof_summary':summary, 'artifact_hashes':files(proof),
            'proof_execution':True, 'funded_application':True}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('disclose-gate', 'compiler', 'compiler-recipe', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--expected-source-sha', required=True)
    parser.add_argument('--cargo-target', required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    require(source['head'] == args.expected_source_sha and
            re.fullmatch('[0-9a-f]{40}', args.expected_source_sha) is not None,
            'proof source differs from exact selected head')
    source_before = tracked_source_hashes()
    require(PROOF_BODY.is_file() and not PROOF_BODY.is_symlink(),
            'dedicated disclose proof body missing')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'fresh proof output required')
    gate_root = args.disclose_gate.resolve(strict=True)
    gate_path = gate_root / 'receipt.json'
    gate = json.loads(gate_path.read_text())
    require(gate['status'] == 'passed' and gate['source'] == source and
            gate['upstream_commit'] == PINNED and
            set(gate['cases']) ==
            {'branch_conditions', 'branch_conditions_in_constructor',
             'unexported_circuit_parameters', 'param_to_ledger_undisclosed',
             'vector_element_condition'},
            'disclose original family gate does not bind this head')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    require(gate['compiler_recipe'] == recipe,
            'disclose gate compiler differs from source-built recipe')
    zkir = args.zkir.resolve(strict=True)
    require(zkir.name == 'zkir', 'official zkir executable required')
    cargo_target = args.cargo_target.resolve(strict=True)
    require(cargo_target.is_dir() and not args.cargo_target.is_symlink(),
            'physical Cargo target required')
    parameter_cache = Path(os.environ.get('MIDNIGHT_PP',
                           str(Path.home() / '.cache/midnight/zk-params')))
    require(parameter_cache.is_absolute() and parameter_cache.is_dir() and
            not parameter_cache.is_symlink(), 'verified proving material absent')
    output.mkdir(parents=True)
    results = {}
    for case in CASES:
        case_target = cargo_target / f'disclose-{case}'
        require(not case_target.exists() and not case_target.is_symlink(),
                f'fresh isolated disclose Cargo target required: {case}')
        case_target.mkdir()
        results[case] = qualified_case(case, gate_root, gate, compiler, scheme, zkir,
                                       case_target, parameter_cache, source_before,
                                       source, output / case)
    for case in CASES:
        bound = json.loads((output / case / 'binding/inputs.json').read_text())
        require(all(physical_sha256(Path(name)) == digest for name, digest in
                    bound['physical_inputs_before_build'].items()),
                f'disclose physical runner input changed after both cases: {case}')
    require(source_state() == source and tracked_source_hashes() == source_before,
            'source changed across two disclose proofs')
    receipt = {**source, 'format':'compact-ledger9-keyword-disclose-proofs/v1',
               'status':'passed', 'platform':os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit':PINNED, 'disclose_gate_sha256':sha256(gate_path),
               'compiler_recipe_sha256':sha256(args.compiler_recipe),
               'proof_body_sha256':sha256(PROOF_BODY),
               'source_count':len(source_before), 'cases':results,
               'limitations':['local Darwin arm64; native Linux qualification remains required']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status':'passed', 'head':source['head'], 'cases':list(results)}))


if __name__ == '__main__':
    main()

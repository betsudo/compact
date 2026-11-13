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

"""Refresh the explicit source snapshot when preparing a new book edition."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess

from editorial import include_commit, include_metadata, select_snapshot

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source-dir', type=Path, required=True)
parser.add_argument('--checkpoint', type=Path, required=True)
parser.add_argument('--pr-metadata', type=Path, required=True, help='Frozen public PR metadata as TSV.')
parser.add_argument('--issue-metadata', type=Path, required=True, help='Frozen public issue metadata as JSON.')
args = parser.parse_args()
SOURCE = args.source_dir.resolve()
CHECKPOINT = args.checkpoint.resolve()
SOURCES = ROOT / 'sources'
SOURCES.mkdir(exist_ok=True)

def git(*args):
    return subprocess.check_output(['git', '-C', str(REPO), *args], text=True).strip()

def digest(data):
    return hashlib.sha256(data).hexdigest()

historical = '51a079bd8598b9decb3df7de92ec10768ec9b8dc'
merged = '05912594fa3c483cf4e7031de83c5f9a103ef95c'
ledger8 = git('rev-parse', 'codex/rust-backend-ast')
local = '7a4fab772ee34c19053ce293d08052b501ce4b50'
jubjub = '6eef58425bb53883c8f3b417b601246820c7d675'
keccak = '30a2d76550beb7d920a92bab2be6b1180243f658'
base = 'eb72a5ab6085bf9b8564cc80c98c973e04825c2d'
oracle = '589c92ef961ce89ac730140e81b76d4c08222a70'
oracle_base = git('merge-base', oracle, base)
checkpoint = json.loads(CHECKPOINT.read_text())
assert checkpoint['version'] == 73
assert checkpoint['head'] == local, 'Choose an explicit edition cutoff before refreshing.'

phases = [
    ('native', 'Native AST milestone', 'ea4cc006f973632c4c7b1fb4569af2b4c93ff38a'),
    ('recording', 'Recorded contracts milestone', '03c03a39a2d39c43c91143b9526ca1c2ffae706c'),
    ('qualified', 'Qualified consumers milestone', '0d7610b2792d60fee5d3713d4eb0ede3c98cc82a'),
    ('effects', 'Compositional recording', '0437c894cd069af475bd8d5b254c24cc73615a09'),
    ('functional', 'Functional Ledger 8', 'c353907d8c323db16eedb35d5a7f0151aab859bb'),
    ('semantics', 'Semantic delivery', ledger8),
    ('ledger9', 'Ledger 9 foundation', merged),
    ('compatibility', 'Upstream compatibility', local),
    ('pending', 'Isolated local follow-ups', keccak),
]
ancestor_sets = {key: set(git('rev-list', ref).splitlines()) for key, _, ref in phases}
oracle_public = git('rev-parse', 'origin/codegen-rust')
published = set(git('rev-list', merged, historical, oracle_public).splitlines())
main = set(git('rev-list', f'{base}..{local}', f'{base}..{merged}', f'{base}..{ledger8}', f'{base}..{keccak}').splitlines())
original = set(git('rev-list', f'{oracle_base}..{oracle}').splitlines()) - main
selected = main | original
lines = git('log', '--all', '--format=%H%x1f%P%x1f%aI%x1f%cI%x1f%s', '--date-order').splitlines()
topics = [
    ('proofs', r'proof|prover|verifier|funded|replay|preimage|zkir'),
    ('types', r'type|uint|field|vector|tuple|alias|literal|cast|struct|enum'),
    ('recording', r'record|effect|fold|observation|trace|ownership'),
    ('crypto', r'schnorr|jubjub|secp|ecdsa|curve|keccak|sha.?512|ed25519|crypto'),
    ('tooling', r'\bci\b|nix|cargo|workflow|cache|package|build|clippy'),
    ('docs', r'^docs|adr|guide|milestone|readme|report'),
    ('state', r'ledger|map|set\b|list\b|merkle|event|counter'),
]
commits = []
omitted = set()
for line in lines:
    sha, parents, authored, committed, subject = line.split('\x1f', 4)
    if sha not in selected:
        continue
    if not include_commit(sha, subject):
        omitted.add(sha)
        continue
    phase = 'oracle' if sha in original else next(
        (key for key, _, _ in phases if sha in ancestor_sets[key]), 'unmapped')
    labels = [key for key, pattern in topics if re.search(pattern, subject, re.I)]
    commits.append(dict(sha=sha, parents=parents.split(), authored=authored, committed=committed,
                        subject=subject, phase=phase, topics=labels,
                        publication='published' if sha in published else 'local',
                        merge=len(parents.split()) > 1))
assert {c['sha'] for c in commits} == selected - omitted
commits.sort(key=lambda c: (c['authored'], c['sha']))
(ROOT/'commits.json').write_text(json.dumps(commits, ensure_ascii=False, separators=(',', ':'))+'\n')
chapter_list = [c for c in json.loads((ROOT/'chapters.json').read_text()) if c['kind'] != 'archive']
archives = [
    ('02-architecture.md', 'The October architecture', 'Frontend, IR, renderer, generated crate and runtime.'),
    ('03-comparison-and-evidence.md', 'Comparing the backend lines', 'The original comparison, with its historical cutoff.'),
    ('06-generated-rust-examples.md', 'Read the generated Rust', 'Source, native path, typed slots and recording.'),
    ('07-runtime-and-macros-domain-model.md', 'Runtime and macro ownership', 'Core types and mechanical derives.'),
    ('08-gas-errors-and-fixes.md', 'Gas, errors and corrected boundaries', 'How the original target assigned errors and costs.'),
    ('04-decisions-and-corrections.md', 'The original decision ledger', 'Corrections and choices as recorded in October.'),
    ('glossary.md', 'A working glossary', 'Vocabulary used throughout the journey.'),
]
for filename, title, subtitle in archives:
    slug='atlas-'+filename.removesuffix('.md')
    raw=git('show', f'{historical}:doc/rust/journey/{filename}')
    raw=raw.replace('github.com/MediaNoxLabs/compact', 'github.com/betsudo/compact')
    def rewrite(match):
        label, href=match.groups()
        if href.startswith(('https:', 'http:', '#', 'mailto:')):
            return match.group(0)
        from posixpath import normpath, join
        target=normpath(join('doc/rust/journey', href))
        return f'[{label}](https://github.com/betsudo/compact/blob/{historical}/{target})'
    raw=re.sub(r'\[([^\]]+)\]\(([^)]+)\)', rewrite, raw)
    raw=re.sub(r'^# .*\n', '', raw, count=1)
    prefix=('> Historical atlas. This chapter preserves the October 14 documentation cutoff. '
            'The main narrative records subsequent deliveries and current limits.\n\n')
    original=f'https://github.com/betsudo/compact/blob/{historical}/doc/rust/journey/{filename}'
    prefix += f'> Source snapshot — [original chapter at 51a079bd]({original}). The implementation cutoff documented in the original is f3cef799 on October 14.\n\n'
    corrections = {
        '03-comparison-and-evidence.md': 'The historical 386-source shorthand refers to 386 proof-required exports. The M2 inventory covered 217 sources, 196 roots and 755 exports, including 369 non-proof exports.',
        '04-decisions-and-corrections.md': 'ADR 0316 records owner-authorized ACC removal after an inventory found no unique witness, struct or ledger mechanism. Its exact application composition remained unqualified.'
    }
    if filename in corrections:
        prefix = '> Editorial correction — '+corrections[filename]+'\n\n'+prefix
    (ROOT/'chapters'/f'{slug}.md').write_text(prefix+raw+'\n')
    chapter_list.append(dict(slug=slug,title=title,phase='Historical atlas',subtitle=subtitle,
                             source=f'chapters/{slug}.md',kind='archive'))
(ROOT/'chapters.json').write_text(json.dumps(chapter_list,indent=2)+'\n')
snapshots={}
for name in ('ledger9-adoption.md','ledger9-upstream-compatibility.md'):
    source=subprocess.check_output(['git','-C',str(SOURCE),'show',f'{local}:doc/rust/plans/{name}'])
    text=select_snapshot(name, source.decode())
    text=re.sub(r'/Users/[^\s\x60"\'<>)]*', '[local-evidence-path]', text)
    prefix=('> Frozen documentation snapshot for the November 13, 2026 book edition. '
            'Entries retain their historical source-specific scope. Later receipts can supersede earlier open items.\n\n')
    data=(prefix+text).encode()
    (SOURCES/name).write_bytes(data)
    snapshots[name]=dict(sha256=digest(data),source_sha256=digest(source))
summaries={}
for key, value in checkpoint['qualification'].items():
    path=value if isinstance(value,str) else value.get('receipt') if isinstance(value,dict) else None
    if not path or not Path(path).is_absolute() or not Path(path).is_file():
        continue
    try:
        data=json.loads(Path(path).read_text())
    except (json.JSONDecodeError,UnicodeDecodeError):
        continue
    summary={'receipt':Path(path).name,'source_sha256':digest(Path(path).read_bytes())}
    for field in ('head','tree','platform','rust','compiler_checks','backend_passed','original_cases',
                  'rust_library_tests','intact_language_admission','standalone_proofs','scope'):
        value=data.get(field)
        if isinstance(value,(str,int,float,bool)):
            if not (isinstance(value,str) and '/Users/' in value):
                summary[field]=value
        elif field=='standalone_proofs' and isinstance(value,dict):
            summary['standalone_proof_cases']=list(value)
    summaries[key]=summary
(SOURCES/'ledger9-delivery-register.json').write_text(json.dumps(summaries,indent=2)+'\n')
pr_rows=[]
for line in args.pr_metadata.read_text().splitlines():
    number,title,state,merged_at,head=line.split('\t')
    pr_rows.append(dict(number=int(number),title=title,state=state,merged_at=merged_at or None,head=head))
(SOURCES/'pull-requests.json').write_text(json.dumps([row for row in pr_rows if include_metadata(row)],indent=2)+'\n')
issues=json.loads(args.issue_metadata.read_text())
(SOURCES/'issues.json').write_text(json.dumps([row for row in issues if include_metadata(row)],indent=2)+'\n')
edition=dict(title='From source to proof',date='2026-11-13',repository='betsudo/compact',
             historical_site=historical,ledger8_cutoff=ledger8,merged_foundation=merged,
             compatibility_cutoff=local,isolated_followups=[jubjub,keccak],checkpoint=73,
             upstream='672bd0f226f47a7d036c5c074ce71fbe1af1925e',release='Compact 0.36.0',
             sdk='Ledger 9.1.0.1',platform='macOS arm64 for local compatibility evidence',
             phases=[dict(id='oracle',title='Original Rust oracle',head=oracle)]+[
                 dict(id=key,title=title,head=head) for key,title,head in phases],
             commit_count=len(commits),main_line_commit_count=sum(c['sha'] in main for c in commits),
             original_only_count=sum(c['sha'] in original for c in commits),
             classification='The catalogue selects commits relevant to the backend journey. Earliest named phase head containing the commit, in listed order. Original-oracle-only commits have their own phase. Topics use subject matching. Dates shown by default are author dates.',
             published_rule='Reachable from the verified working-line, Pages or original codegen-rust public head at the cutoff. Other records are labeled local.',
             published_heads=dict(working_line=merged,documentation=historical,original_oracle=oracle_public),
             snapshots=snapshots,chapter_count=len(chapter_list))
(SOURCES/'edition.json').write_text(json.dumps(edition,indent=2)+'\n')
print(len(commits),'commits;',len(chapter_list),'chapters;',len(summaries),'qualification summaries')

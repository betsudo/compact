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

"""Freeze commit descriptions and render readable references in the book."""
from html import escape
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import urlsplit

from editorial import include_commit, select_message

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
DATA = ROOT / 'sources' / 'revisions.json'
SHA = r'[0-9a-f]{7,40}'
REFERENCE = re.compile(rf'(?<![\w/]){SHA}(?:\.{{2,3}}{SHA})?(?!\w)')
COMMIT_URL = re.compile(rf'^/([^/]+)/compact/commit/({SHA})/?$')
REPOSITORY_URL = 'https://github.com/'

def git(*args, input=None):
    return subprocess.check_output(['git', '-C', str(REPO), *args], text=True, input=input)

def freeze():
    """Collect metadata from the edition's Git objects, without fetching at build time."""
    catalogue = {row['sha']: row for row in json.loads((ROOT/'commits.json').read_text())}
    iteration_notes = {
        row['sha']: row['explanation']
        for row in json.loads((ROOT/'sources/iterations.json').read_text())
    }
    edition = json.loads((ROOT/'sources/edition.json').read_text())
    iteration_notes.update({
        'f3cef7992d2c837be7950e25e8f5f8065f7f7df0': 'The historical storybook froze this local prototype checkpoint while final checked-effect qualification was still open.',
        '6eef58425bb53883c8f3b417b601246820c7d675': 'Ports the upstream Jubjub identity-key assertion. The local qualification covers generated Rust and TypeScript, fresh proofs, funded applications and replay refusal.',
        '30a2d76550beb7d920a92bab2be6b1180243f658': 'Ports the upstream opaque-input Keccak diagnostic. Direct TypeScript, Rust and paired-target probes reproduced the expected source error.'
    })
    texts = [p.read_text() for p in (ROOT/'chapters').glob('*.md')]
    texts += list(iteration_notes.values())
    texts += [(REPO/'site/rust-ast-storybook/index.html').read_text(), json.dumps(edition)]
    # Include URL targets as well as visible labels.
    tokens = set(re.findall(rf'(?<![\w]){SHA}(?!\w)', '\n'.join(texts)))
    revisions = set(catalogue)
    for token in sorted(tokens):
        result = subprocess.run(
            ['git', '-C', str(REPO), 'rev-parse', '--verify', token+'^{commit}'],
            text=True, capture_output=True,
        )
        if result.returncode == 0:
            revisions.add(result.stdout.strip())
    public = set(git('rev-list', *edition['published_heads'].values()).splitlines())
    upstream = set(git('rev-list', edition['upstream']).splitlines())
    log = git('log', '--no-walk', '--stdin', '-z', '--format=%H%x00%aI%x00%cI%x00%s%x00%b',
              input='\n'.join(sorted(revisions))+'\n')
    fields = log.removesuffix('\0').split('\0')
    assert len(fields) == 5 * len(revisions)
    omitted = {
        fields[index] for index in range(0, len(fields), 5)
        if not include_commit(fields[index], fields[index + 3])
    }
    records = {}
    for index in range(0, len(fields), 5):
        sha, authored, committed, subject, message = fields[index:index+5]
        if sha in omitted:
            continue
        repository = 'betsudo/compact' if sha in public else (
            'LFDT-Minokawa/compact' if sha in upstream else None
        )
        if sha in (edition['upstream'], 'eb72a5ab6085bf9b8564cc80c98c973e04825c2d'):
            repository = 'LFDT-Minokawa/compact'
        # Narrative notes describe the larger iteration, even when its final
        # commit subject describes only the last change.
        body = ''
        if repository:
            body = re.sub(r'(?im)^(?:signed-off-by|co-authored-by|assisted-by):.*\n?', '', message)
            body = re.sub(r'/Users/[^\s<>]+', '[local path]', body).strip()
            body = select_message(body, omitted)
        records[sha] = dict(sha=sha, subject=subject, authored=authored, committed=committed,
                            url=f'{REPOSITORY_URL}{repository}/commit/{sha}' if repository else None,
                            description=iteration_notes.get(sha, ''), message=body,
                            in_catalogue=sha in catalogue)
    assert records.keys() == revisions - omitted
    DATA.write_text(json.dumps(records, ensure_ascii=False, sort_keys=True, separators=(',', ':'))+'\n')
    print(f'Frozen descriptions for {len(records)} commits')

class References:
    def __init__(self, records):
        self.records = records
        self.prefixes = {}
        for sha in records:
            for length in range(7, len(sha)+1):
                prefix = sha[:length]
                self.prefixes[prefix] = sha if prefix not in self.prefixes else None

    def lookup(self, token):
        return self.records.get(self.prefixes.get(token))

    @staticmethod
    def button(record):
        return (
            f' <button type="button" class="commit-details" data-commit="{record["sha"]}" '
            f'aria-haspopup="dialog" aria-label="Commit details for {escape(record["subject"], quote=True)}">'
            'Details</button>'
        )

    def link(self, record, prefix, details):
        href = record['url'] or f'{prefix}revisions.html#commit-{record["sha"]}'
        link = f'<a class="commit-title" href="{escape(href, quote=True)}">{escape(record["subject"])}</a>'
        return f'<span class="commit-reference">{link}{self.button(record) if details else ""}</span>'

    def rewrite(self, markup, prefix='./', details=True):
        parser = ReferenceHTML(self, prefix, details)
        parser.feed(markup)
        parser.close()
        return ''.join(parser.output)

class ReferenceHTML(HTMLParser):
    """Replace visible revision tokens while retaining links and code samples."""
    def __init__(self, references, prefix, details):
        super().__init__(convert_charrefs=False)
        self.references = references
        self.prefix = prefix
        self.details = details
        self.output = []
        self.protected = None
        self.in_link = False
        self.link_record = None

    def handle_starttag(self, tag, attrs):
        self.output.append(self.get_starttag_text())
        if tag in ('pre', 'script', 'style', 'textarea'):
            self.protected = tag
        if tag == 'details' and 'revision-identity' in dict(attrs).get('class', '').split():
            self.protected = tag
        if tag == 'a':
            self.in_link = True
            href = urlsplit(dict(attrs).get('href', ''))
            match = COMMIT_URL.fullmatch(href.path) if href.netloc == 'github.com' else None
            self.link_record = self.references.lookup(match[2]) if match else None

    def handle_startendtag(self, tag, attrs):
        self.output.append(self.get_starttag_text())

    def handle_endtag(self, tag):
        self.output.append(f'</{tag}>')
        if tag == self.protected:
            self.protected = None
        if tag == 'a':
            if self.details and self.link_record:
                self.output.append(self.references.button(self.link_record))
            self.in_link = False
            self.link_record = None

    def handle_data(self, data):
        if self.protected:
            self.output.append(data)
            return

        def replace(match):
            text = match[0]
            parts = re.split(r'\.{2,3}', text)
            records = [self.references.lookup(part) for part in parts]
            if not all(records):
                return text
            if self.in_link:
                self.link_record = self.link_record or records[-1]
                return ' → '.join(escape(record['subject']) for record in records)
            return ' → '.join(self.references.link(record, self.prefix, self.details) for record in records)

        self.output.append(REFERENCE.sub(replace, data))

    def handle_entityref(self, name):
        self.output.append(f'&{name};')

    def handle_charref(self, name):
        self.output.append(f'&#{name};')

    def handle_comment(self, data):
        self.output.append(f'<!--{data}-->')

    def handle_decl(self, decl):
        self.output.append(f'<!{decl}>')

if __name__ == '__main__':
    freeze()

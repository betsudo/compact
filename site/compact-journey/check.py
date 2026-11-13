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

"""Validate the published book, its source identities and local navigation."""
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit, unquote
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
BOOK = REPO/'docs/book'

class Page(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.ids = []
        self.links = []
        self.scripts = []
        self.protected = None
        self.visible_text = []
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag in ('pre','script','style') or (tag == 'details' and attrs.get('class') == 'revision-identity'):
            self.protected = tag
        if 'id' in attrs:
            self.ids.append(attrs['id'])
        if tag in ('a', 'link') and attrs.get('href'):
            self.links.append(attrs['href'])
        if tag in ('script', 'img') and attrs.get('src'):
            self.links.append(attrs['src'])
        if tag == 'script' and attrs.get('src'):
            self.scripts.append(attrs['src'])

    def handle_endtag(self, tag):
        if tag == self.protected:
            self.protected = None

    def handle_data(self, data):
        if not self.protected:
            self.visible_text.append(data)

def main():
    failures = []
    pages = {path: Page(path.read_text()) for path in BOOK.rglob('*.html')}
    external = set()
    checked = 0
    for path, page in pages.items():
        for key, count in Counter(page.ids).items():
            if count > 1:
                failures.append(f'Duplicate id {key} in {path.relative_to(BOOK)}')
        for href in page.links:
            parsed = urlsplit(href)
            if parsed.scheme or parsed.netloc:
                external.add(href)
                continue
            target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if target.is_dir():
                target /= 'index.html'
            if not target.is_file():
                failures.append(f'Missing {href} in {path.relative_to(BOOK)}')
                continue
            if parsed.fragment and target in pages and unquote(parsed.fragment) not in pages[target].ids:
                failures.append(f'Missing fragment {href} in {path.relative_to(BOOK)}')
            checked += 1
        if re.search(r'/Users/|file://|localhost|127\.0\.0\.1', path.read_text()):
            failures.append(f'Workstation reference in {path.relative_to(BOOK)}')
        if any(urlsplit(src).scheme for src in page.scripts):
            failures.append(f'Remote script in {path.relative_to(BOOK)}')

    for stylesheet in (BOOK/'assets').glob('*.css'):
        for href in re.findall(r'url\(([^)]+)\)', stylesheet.read_text()):
            href=href.strip('"\'')
            if not urlsplit(href).scheme:
                assert (stylesheet.parent/href).is_file(), href

    commits = json.loads((ROOT/'commits.json').read_text())
    edition = json.loads((ROOT/'sources/edition.json').read_text())
    chapters = json.loads((ROOT/'chapters.json').read_text())
    assert len(commits) == edition['commit_count'] == len({c['sha'] for c in commits})
    assert len(chapters) == edition['chapter_count']
    assert all(re.fullmatch('[0-9a-f]{40}', c['sha']) for c in commits)
    assert all(c['phase'] in {p['id'] for p in edition['phases']} for c in commits)
    assert all(c['publication'] in ('local','published') for c in commits)
    assert len(pages) == len(chapters) + 7
    iterations=json.loads((ROOT/'sources/iterations.json').read_text())
    identities={commit['sha'] for commit in commits}
    assert all(row['sha'] in identities for row in iterations)
    assert len({row['sha'] for row in iterations})==len(iterations)
    revisions = json.loads((ROOT/'sources/revisions.json').read_text())
    assert identities <= revisions.keys()
    assert all(sha == row['sha'] and row['subject'] for sha, row in revisions.items())
    prefixes = {sha[:length] for sha in revisions for length in range(7, len(sha)+1)}
    reading_pages = dict(pages)
    reading_pages[BOOK.parent/'index.html'] = Page((BOOK.parent/'index.html').read_text())
    for path, page in reading_pages.items():
        bare = prefixes.intersection(re.findall(r'\b[0-9a-f]{7,40}\b', ' '.join(page.visible_text)))
        if bare:
            failures.append(f'Bare revision labels in {path.name}: {sorted(bare)}')
    for name, identity in edition['snapshots'].items():
        assert hashlib.sha256((ROOT/'sources'/name).read_bytes()).hexdigest() == identity['sha256']
    for path in BOOK.rglob('*'):
        if path.is_file() and path.suffix in ('.json','.md','.html','.css','.js'):
            assert '/Users/' not in path.read_text(), path
    manifest = json.loads((BOOK/'build-manifest.json').read_text())
    for name, sha in manifest['files'].items():
        assert hashlib.sha256((BOOK/name).read_bytes()).hexdigest() == sha, name

    # Verify immutable commit and blob references against the available repository graph.
    verified = set()
    for href in sorted(external):
        match = re.match(r'https://github\.com/(?:betsudo|MediaNoxLabs)/compact/(commit|blob|tree)/([^/#?]+)(?:/([^#?]+))?', href)
        if not match:
            continue
        kind, ref, location = match.groups()
        if not re.fullmatch('[0-9a-f]{7,40}', ref):
            continue
        identity = ref + (':'+unquote(location) if kind in ('blob','tree') and location else '^{commit}')
        if identity in verified:
            continue
        result = subprocess.run(['git','-C',str(REPO),'cat-file','-e',identity],capture_output=True,text=True)
        if result.returncode:
            failures.append(f'Unresolved source {href}')
        else:
            verified.add(identity)
    report = dict(chapters=len(chapters),pages=len(pages),commits=len(commits),
                  local_links_checked=checked,immutable_source_identities_checked=len(verified),
                  failures=failures)
    print(json.dumps(report,indent=2))
    if failures:
        raise SystemExit(1)

if __name__ == '__main__':
    main()

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

"""Build the frozen Markdown book into the existing GitHub Pages directory."""
from pathlib import Path
import hashlib
import html
import json
import re
import shutil
import os
import tempfile
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
import markdown
from revisions import DATA, References

ROOT = Path(__file__).resolve().parent
OUTPUT = ROOT.parents[1] / 'docs' / 'book'
CHAPTERS = json.loads((ROOT / 'chapters.json').read_text())
EDITION = json.loads((ROOT / 'sources' / 'edition.json').read_text())
REFERENCES = References(json.loads(DATA.read_text()))
ASSET_VERSIONS = {
    name: hashlib.sha256((ROOT/'assets'/name).read_bytes()).hexdigest()[:12]
    for name in ('book.css','book.js')
}

def escape(value):
    return html.escape(str(value), quote=True)

def chapter_href(chapter):
    return f"chapters/{chapter['slug']}.html"

def navigation(prefix, selected=''):
    groups = {}
    for chapter in CHAPTERS:
        groups.setdefault(chapter['phase'], []).append(chapter)
    parts = ['<nav class="chapter-nav" aria-label="Book chapters">']
    for phase, chapters in groups.items():
        parts.append(f'<h2>{escape(phase)}</h2><ul>')
        for chapter in chapters:
            current = ' aria-current="page"' if chapter['slug'] == selected else ''
            parts.append(f'<li><a href="{prefix}{chapter_href(chapter)}" data-chapter="{chapter["slug"]}"{current}>{escape(chapter["title"])}</a></li>')
        parts.append('</ul>')
    parts.append('</nav>')
    return ''.join(parts)

def shell(title, body, *, prefix='./', selected='', toc='', description='The Compact engineering journey from Ledger 8 TypeScript to the Ledger 9 Rust AST backend.'):
    body = REFERENCES.rewrite(body, prefix)
    toc = REFERENCES.rewrite(toc, prefix, details=False)
    return f'''<!doctype html>
<html lang="en" data-theme="light">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="description" content="{escape(description)}">
<meta name="theme-color" content="#f5f2eb">
<title>{escape(title)} · The Compact journey</title>
<link rel="stylesheet" href="{prefix}assets/fonts.css">
<link rel="stylesheet" href="{prefix}assets/book.css?v={ASSET_VERSIONS["book.css"]}">
<script defer src="{prefix}assets/book.js?v={ASSET_VERSIONS["book.js"]}"></script>
<noscript><style>
.header-tools,.chapter-actions button,.commit-filters,.table-pagination,#commit-status,#chapter-search,label[for="chapter-search"],#print-book,.commit-details {{display:none}}
@media(max-width:740px){{.sidebar{{position:static;width:auto;transform:none;visibility:visible;pointer-events:auto;max-height:none}}.sidebar-tools{{display:grid}}}}
</style></noscript>
</head>
<body data-book-root="{prefix}" data-chapter-id="{selected}">
<a class="skip-link" href="#content">Skip to content</a>
<header class="masthead">
<a class="wordmark" href="{prefix}index.html">COMPACT <span>ENGINEERING BOOK</span></a>
<nav aria-label="Primary"><a href="{prefix}index.html">The story</a><a href="{prefix}iterations.html">Iterations</a><a href="{prefix}architecture.html">The pipeline</a><a href="{prefix}commits.html">Commit explorer</a><a href="{prefix}sources.html">Sources</a></nav>
<div class="header-tools"><button id="theme-toggle" type="button" aria-label="Switch to dark theme">Dark</button><button id="menu-toggle" type="button" aria-expanded="false" aria-controls="book-sidebar">Chapters</button></div>
</header>
<aside class="sidebar" id="book-sidebar">
<a class="sidebar-title" href="{prefix}index.html">From source to proof</a>
<nav class="sidebar-tools" aria-label="Book tools"><a href="{prefix}index.html">The story</a><a href="{prefix}iterations.html">Iterations</a><a href="{prefix}architecture.html">The pipeline</a><a href="{prefix}commits.html">Commit explorer</a><a href="{prefix}sources.html">Sources</a></nav>
<label for="chapter-search">Find a chapter</label><input id="chapter-search" type="search" placeholder="Types, proofs, Ledger 9…" autocomplete="off">
<p id="chapter-search-status" class="sr-only" role="status"></p>
{navigation(prefix, selected)}
<div class="sidebar-footer"><a href="{prefix}print.html">Print / save the complete book</a><span id="reading-progress"></span><small>Edition · 13 November 2026</small></div>
</aside>
<main id="content" tabindex="-1">{body}</main>
{f'<aside class="on-this-page"><h2>In this chapter</h2>{toc}</aside>' if toc else ''}
<dialog class="commit-panel" id="commit-panel" aria-labelledby="commit-panel-title">
<form method="dialog"><button type="submit" autofocus aria-label="Close commit details">Close</button></form>
<h2 id="commit-panel-title">Commit details</h2>
<div id="commit-panel-content" aria-live="polite"></div>
</dialog>
<footer class="footer"><span>Compact · From source to proof</span><span>Ledger 8 → Ledger 9 · Evidence frozen on 13 November 2026</span><a href="{prefix}sources.html">Edition and sources</a></footer>
</body></html>'''

def parse(chapter):
    text = (ROOT / chapter['source']).read_text()
    parser = markdown.Markdown(extensions=['fenced_code', 'tables', 'toc', 'sane_lists'])
    content = parser.convert(text)
    words = len(re.findall(r'\b\w+\b', text))
    return content, parser.toc, words

def write(name, text):
    path = OUTPUT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)

def build():
    global OUTPUT
    destination = OUTPUT
    if destination.is_symlink():
        raise RuntimeError("Refusing a symlink publication directory")
    staging = tempfile.TemporaryDirectory(prefix=".compact-journey-", dir=destination.parent)
    OUTPUT = Path(staging.name)/"new"
    OUTPUT.mkdir()
    (OUTPUT/'assets').mkdir()
    shutil.copytree(ROOT/'assets', OUTPUT/'assets', dirs_exist_ok=True)
    (OUTPUT/'sources').mkdir()
    for name in ('edition.json','ledger9-adoption.md','ledger9-upstream-compatibility.md',
                 'ledger9-delivery-register.json','pull-requests.json','issues.json','iterations.json','revisions.json'):
        shutil.copyfile(ROOT/'sources'/name, OUTPUT/'sources'/name)
    (OUTPUT/'markdown').mkdir()
    for chapter in CHAPTERS:
        shutil.copyfile(ROOT/chapter['source'], OUTPUT/'markdown'/Path(chapter['source']).name)
    shutil.copyfile(ROOT/'commits.json', OUTPUT/'commits.json')
    search = []
    print_chapters = []
    total_words = 0
    for index, chapter in enumerate(CHAPTERS):
        content, toc, words = parse(chapter)
        total_words += words
        minutes = max(1, round(words / 220))
        before = CHAPTERS[index - 1] if index else None
        after = CHAPTERS[index + 1] if index + 1 < len(CHAPTERS) else None
        links = []
        if before:
            links.append(f'<a class="previous" href="{before["slug"]}.html"><small>Previous chapter</small>{escape(before["title"])}</a>')
        if after:
            links.append(f'<a class="next" href="{after["slug"]}.html"><small>Next chapter</small>{escape(after["title"])} <span aria-hidden="true">→</span></a>')
        body = f'''<article class="reader">
<header class="chapter-heading"><p class="eyebrow">{escape(chapter["phase"])} <span>· {minutes} min read</span></p>
<h1>{escape(chapter["title"])}</h1><p class="deck">{escape(chapter["subtitle"])}</p></header>
<div class="prose">{content}</div>
<div class="chapter-actions"><button type="button" id="mark-read" aria-pressed="false">Mark chapter as read</button><a href="../markdown/{Path(chapter["source"]).name}">Read the frozen Markdown source</a></div>
<nav class="chapter-pagination" aria-label="Chapter navigation">{"".join(links)}</nav>
</article>'''
        write(chapter_href(chapter), shell(chapter['title'], body, prefix='../', selected=chapter['slug'], toc=toc, description=chapter['subtitle']))
        search.append(dict(slug=chapter['slug'], title=chapter['title'], phase=chapter['phase'],
                           text=re.sub(r'<[^>]+>', ' ', content), words=words, kind=chapter['kind']))
        printable = re.sub(r'href="\.\./sources/', 'href="sources/', content)
        # Prefix each article's fragment identifiers in the combined print document.
        printable = re.sub(r'id="([^"]+)"', lambda m: f'id="{chapter["slug"]}-{m[1]}"', printable)
        printable = re.sub(r'href="#([^"]+)"', lambda m: f'href="#{chapter["slug"]}-{m[1]}"', printable)
        print_chapters.append(f'<article class="print-chapter"><p class="eyebrow">{escape(chapter["phase"])}</p><h1>{escape(chapter["title"])}</h1><div class="prose">{printable}</div></article>')
    write('search.json', json.dumps(search, ensure_ascii=False))
    chapter_cards = ''.join(f'''<a class="chapter-card" href="{chapter_href(c)}"><p class="eyebrow">{escape(c["phase"])}</p><h3>{escape(c["title"])}</h3><p>{escape(c["subtitle"])}</p><span class="card-link">Read chapter <span aria-hidden="true">↗</span></span></a>''' for c in CHAPTERS if c['kind'] == 'narrative')
    phases = [
        ('Ledger 8 reference','A stable protocol family and an independent behavioral reference.','typescript-and-fixed-ground'),
        ('Native Rust AST','Typed IR, checked lowering, structured syntax and ordinary Rust execution.','a-closed-rust-boundary'),
        ('Recorded contracts','Ordered effects, replay, installed operations and proof preparation.','recording-is-an-execution'),
        ('0.3.0 closure','Scoped applications, consumer qualification and explicit acceptance.','closing-zero-three'),
        ('Compositional effects','Operations and ownership replace growing shape catalogues.','operations-instead-of-shapes'),
        ('Ledger 9 foundation','A new SDK graph, ABI 54, Rust 1.99 and funded transactions.','ledger9-is-a-protocol-migration'),
        ('Compatibility work','Cryptography, wire values, events and the remaining corpus.','porting-the-cryptographic-surface'),
    ]
    timeline = ''.join(f'<li><a href="chapters/{slug}.html"><strong>{escape(title)}</strong><span>{escape(text)}</span></a></li>' for title,text,slug in phases)
    body = f'''<section class="book-hero">
<p class="eyebrow">AN ENGINEERING HISTORY · LEDGER 8 → LEDGER 9</p>
<h1>From source<br>to <em>proof.</em></h1>
<p class="hero-deck">The story of making Compact a typed Rust target. The decisions, the failed assumptions, and the tests that turned a compiler experiment into a protocol migration.</p>
<div class="hero-actions"><a class="button primary" href="{chapter_href(CHAPTERS[0])}">Start reading <span aria-hidden="true">→</span></a><a class="button" href="architecture.html">Follow a circuit</a><a id="continue-reading" class="resume-link" hidden href="#">Continue reading</a></div>
<div class="edition-strip"><span>{len(CHAPTERS)} chapters</span><span>{total_words:,} words</span><span>{EDITION["commit_count"]:,} indexed commits</span><span>13 November 2026 edition</span></div><p class="download"><a href="https://betsudo.github.io/compact/book/compact-journey.zip" download>Download the book and Markdown chapters</a></p>
</section>
<section class="home-section"><div class="section-heading"><p class="eyebrow">THE THROUGH LINE</p><h2>A new owner for every<br>hidden assumption.</h2><p>The chapters connect the reason for each change to its implementation and the evidence that followed.</p></div><ol class="timeline">{timeline}</ol></section>
<section class="status-panel"><h2>Where the story stands</h2><div><p><strong>Ledger 9 foundation merged.</strong> PR #725 brought the SDK and toolchain migration into the working branch.</p><p><strong>Upstream compatibility continues locally.</strong> Recent cryptography, event, serialization and proof results have named arm64 source revisions. Full corpus, native Linux qualification and generated two-contract execution remain open.</p><a href="chapters/the-next-reader-inherits-a-map.html">Read the current boundary →</a></div></section>
<section class="home-section"><div class="section-heading"><p class="eyebrow">CONTENTS</p><h2>The complete story</h2><p>Read in sequence or enter through the engineering problem that interests you.</p></div><div class="chapter-grid">{chapter_cards}</div></section>
<section class="atlas-promo"><p class="eyebrow">THE HISTORICAL ATLAS</p><h2>The architecture as it was understood then.</h2><p>Seven preserved chapters explain generated Rust, runtime ownership, macros, gas and errors at the October 14 cutoff.</p><a class="button" href="chapters/{CHAPTERS[16]["slug"]}.html">Open the atlas →</a><a class="text-link" href="https://betsudo.github.io/compact/">Visit the original interactive storybook</a></section>'''
    write('index.html', shell('From source to proof', body))
    options=''.join(f'<option value="{p["id"]}">{escape(p["title"])}</option>' for p in EDITION['phases'])
    body=f'''<article class="wide-reader"><header class="chapter-heading"><p class="eyebrow">THE GRANULAR RECORD</p><h1>Commit explorer</h1><p class="deck">Find an iteration, trace a repair, or browse the steps inside a milestone.</p></header>
<div class="prose"><p>This frozen catalogue contains {EDITION["commit_count"]:,} unique commits across the selected AST, migration, follow-up and original-oracle histories. Phase grouping follows Git reachability. Topic tags use commit-subject matching. Author and committer dates are retained.</p><p>Published links are limited to commits reachable from the verified working-line, Pages or original-oracle public heads. Local records retain their full identity here.</p></div>
<form class="commit-filters" id="commit-filters"><label>Search<input id="commit-query" type="search" placeholder="Hash, subject or topic" autocomplete="off"></label><label>Phase<select id="commit-phase"><option value="">All phases</option>{options}</select></label><label>Topic<select id="commit-topic"><option value="">All topics</option>{"".join(f'<option>{v}</option>' for v in ['proofs','types','recording','crypto','tooling','docs','state'])}</select></label><label>Publication<select id="commit-publication"><option value="">All records</option><option value="published">Published history</option><option value="local">Local at cutoff</option></select></label><label>Order<select id="commit-order"><option value="oldest">Oldest author date first</option><option value="newest">Newest author date first</option></select></label><button type="reset">Clear filters</button></form>
<p id="commit-status" class="results-status" role="status">Loading commit catalogue…</p>
<div class="table-scroll"><table class="commit-table"><thead><tr><th>Authored</th><th>Commit / iteration</th><th>Phase</th><th>Scope</th></tr></thead><tbody id="commit-results"></tbody></table></div>
<div class="table-pagination"><button id="commit-prev" type="button">← Previous</button><span id="commit-page"></span><button id="commit-next" type="button">Next →</button></div>
<noscript><p>The interactive filters need JavaScript. <a href="commits.json">Download the complete JSON catalogue</a> or read the <a href="sources/edition.json">edition manifest</a>.</p></noscript>
<p class="download"><a href="commits.json" download>Download the commit catalogue</a> · <a href="sources/edition.json">Read the classification rules</a></p></article>'''
    write('commits.html', shell('Commit explorer',body))
    body='''<article class="wide-reader"><header class="chapter-heading"><p class="eyebrow">FROM ONE CIRCUIT TO A LEDGER TRANSITION</p><h1>Follow the pipeline</h1><p class="deck">Select a stage to see its owner, the evidence it creates, and the next boundary to check.</p></header>
<div class="pipeline" role="group" aria-label="Pipeline stages"></div><section id="pipeline-detail" class="pipeline-detail" aria-live="polite"></section>
<div class="prose"><h2>The shared semantic thread</h2><p>Compact source flows through shared frontend analysis into a typed internal model. Checked lowering assigns legal target operations. The Rust AST renderer constructs syntax, while the runtime delegates protocol behavior to the ledger SDK.</p><p>Native execution and recorded execution expose different results from the same admitted behavior. Proof preparation binds the observed operation, verifier and statement. Proving and ledger application add their own acceptance gates.</p><h2>What type safety contributes</h2><p>A separate Rust carrier can prevent an ordinary K1 call from accepting a P-256 value. Checked decoding can refuse malformed bytes at a host boundary. Neither replaces validation of the installed operation or cryptographic verification of a statement.</p><a href="chapters/a-proof-is-one-stage.html">Read the complete evidence guide →</a></div>
<noscript><p>Read the <a href="chapters/a-proof-is-one-stage.html">pipeline chapter</a> for the complete text without JavaScript.</p></noscript></article>'''
    write('architecture.html',shell('Follow the pipeline',body))
    iterations=json.loads((ROOT/'sources/iterations.json').read_text())
    sections=[]
    for phase in EDITION['phases']:
        rows=[row for row in iterations if row['phase']==phase['id']]
        if not rows:
            continue
        entries=[]
        for row in rows:
            record = REFERENCES.records[row['sha']]
            url = record['url'] or f'revisions.html#commit-{row["sha"]}'
            label = 'View commit on GitHub' if record['url'] else 'View local revision'
            entries.append(f'<details class="iteration"><summary><span class="iteration-date">{row["date"]}</span>{escape(row["title"])}</summary><p>{escape(row["explanation"])}</p><p><a href="{url}">{label}</a></p></details>')
        sections.append(f'<section class="iteration-phase"><h2>{escape(phase["title"])}</h2>{"".join(entries)}</section>')
    body=f'<article class="wide-reader"><header class="chapter-heading"><p class="eyebrow">MILESTONES TO IMPLEMENTATION STEPS</p><h1>The iteration ledger</h1><p class="deck">Open a step to read what changed, the problem it addressed, and the evidence boundary.</p></header><div class="prose"><p>These {len(iterations)} representative implementation and acceptance checkpoints connect the narrative to the commit graph. A checkpoint can summarize several commits. Its explanation records that distinction. The full catalogue remains available in the commit explorer.</p></div>{"".join(sections)}</article>'
    write('iterations.html',shell('The iteration ledger',body))
    local_records = [record for record in REFERENCES.records.values() if not record['url']]
    local_cards = []
    for record in sorted(local_records, key=lambda row: row['authored'], reverse=True):
        description = record['description'] or 'Local revision retained in the edition’s Git catalogue.'
        local_cards.append(f'<section id="commit-{record["sha"]}"><h2>{escape(record["subject"])}</h2>'
                           f'<p>{escape(description)}</p><p>Authored {record["authored"][:10]}</p>'
                           f'<details class="revision-identity"><summary>Full commit identity</summary>'
                           f'<code>{record["sha"]}</code></details></section>')
    local_body = '<article class="reader"><header class="chapter-heading"><h1>Local revisions</h1><p class="deck">Commits preserved in the edition’s local evidence.</p></header><div class="prose">' + ''.join(local_cards) + '</div></article>'
    write('revisions.html', shell('Local revisions', local_body))
    public_sources=''.join(f'<li><a href="sources/{escape(name)}">{escape(name)}</a></li>' for name in ('ledger9-adoption.md','ledger9-upstream-compatibility.md','ledger9-delivery-register.json','pull-requests.json','issues.json','iterations.json','edition.json'))
    body=f'''<article class="reader"><header class="chapter-heading"><p class="eyebrow">HOW THIS EDITION WAS MADE</p><h1>Sources and boundaries</h1><p class="deck">A public, reproducible record of the material behind the story.</p></header>
<div class="prose"><h2>Primary sources</h2><p>The narrative combines verified Git history, repository ADRs, qualification plans and reports, public issue and pull-request metadata, and the owner's Obsidian engineering journal. Private notes are summarized. Complete witness data, keys and workstation paths are excluded from the public edition.</p>
<h2>Frozen source identities</h2><table><thead><tr><th>Source</th><th>Revision</th></tr></thead><tbody>
<tr><td>Existing storybook</td><td><code>{EDITION["historical_site"]}</code></td></tr>
<tr><td>Historical local Ledger 8 line</td><td><code>{EDITION["ledger8_cutoff"]}</code></td></tr>
<tr><td>Merged Ledger 9 foundation</td><td><code>{EDITION["merged_foundation"]}</code></td></tr>
<tr><td>Local compatibility cutoff</td><td><code>{EDITION["compatibility_cutoff"]}</code></td></tr>
<tr><td>Mainstream reference</td><td><code>{EDITION["upstream"]}</code></td></tr></tbody></table>
<h2>Public source snapshots</h2><ul>{public_sources}</ul>
<h2>Interpret the evidence</h2><p>Implemented means the code exists at a named revision. Local qualification records a passing local gate. Independent coding-agent review is a second implementation review. Milestone closure refers to an explicitly accepted scope. Publication status in the commit catalogue records reachability from the edition’s recorded public heads at this cutoff.</p><p>The older atlas is presented with its October cutoff. It contains then-open work that later chapters resolve. Current local arm64 receipts establish their named cases and source revisions. Required native Linux, intact newer-language admission and full-corpus compatibility remain open where identified.</p>
<h2>Editorial grouping</h2><p>{escape(EDITION["classification"])}</p><p>Git dates can differ after rebasing or a signed history rewrite. The catalogue retains both author and committer dates. A phase is an explanatory grouping, not a release declaration.</p>
<h2>Preserved corrections</h2><ul><li>The original oracle is another Ledger 8 line, with an earlier shared ancestor than the selected AST base.</li><li>The 0.3.0 closeout retains composite evidence and the failed full run.</li><li>Forge's local reuse measurements do not quantify whole-CI savings.</li><li>A pragma-only derivative has a different source identity from the original fixture.</li></ul>
<h2>Maintain the book</h2><p>Markdown chapters, frozen records, static assets and a small Python builder live in <a href="https://github.com/betsudo/compact/tree/rust-ast-docs/site/compact-journey">site/compact-journey</a>. The site is pre-rendered and published through the existing Pages branch. Updating it requires no compiler, key generation or additional CI workflow.</p>
</div></article>'''
    write('sources.html',shell('Sources and boundaries',body))
    body='<div class="print-controls"><h1>The complete book</h1><p>Use your browser’s Print command to save a PDF. Chapter navigation and controls are hidden in print.</p><button type="button" id="print-book">Print / save PDF</button></div>'+''.join(print_chapters)
    write('print.html',shell('Complete book',body))
    # Fixed ZIP metadata makes repeated builds reproducible in the same Python/zlib environment.
    with ZipFile(OUTPUT/'compact-journey.zip','w') as archive:
        paths=[p for p in sorted(OUTPUT.rglob('*')) if p.is_file() and p.name not in ('compact-journey.zip','build-manifest.json')]
        entries=[(str(p.relative_to(OUTPUT)),p) for p in paths]
        for name,path in entries:
            item=ZipInfo(name,date_time=(1980,1,1,0,0,0))
            item.compress_type=ZIP_DEFLATED
            item.external_attr=0o100644 << 16
            archive.writestr(item,path.read_bytes())
    manifest={}
    for path in sorted(OUTPUT.rglob('*')):
        if path.is_file() and path.name != 'build-manifest.json':
            manifest[str(path.relative_to(OUTPUT))]=hashlib.sha256(path.read_bytes()).hexdigest()
    write('build-manifest.json',json.dumps(dict(chapters=len(CHAPTERS),words=total_words,commits=EDITION['commit_count'],files=manifest),indent=2)+'\n')
    previous = Path(staging.name)/"previous"
    if destination.exists():
        os.replace(destination, previous)
    try:
        os.replace(OUTPUT, destination)
    except BaseException:
        if previous.exists():
            os.replace(previous, destination)
        raise
    OUTPUT = destination
    staging.cleanup()
    # Keep the historical overview's source intact and update its reading view.
    overview = (ROOT.parent/'rust-ast-storybook/index.html').read_text()
    (OUTPUT.parent/'index.html').write_text(REFERENCES.rewrite(overview, './book/', details=False))
    print(f'Built {len(CHAPTERS)} chapters, {total_words:,} words and {EDITION["commit_count"]:,} commits in {OUTPUT}')

if __name__=='__main__':
    build()

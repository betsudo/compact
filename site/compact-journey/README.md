# From source to proof

A pre-rendered interactive book about Compact's journey from the Ledger 8
TypeScript reference path to a Ledger 9 Rust AST backend.

The November 13, 2026 edition contains 24 chapters, a searchable commit
catalogue, 36 curated iteration checkpoints, a pipeline explorer, local reading progress, light/dark themes,
and a complete print view. The retained Ledger 8 architecture overview stays available at the Pages root.
The book’s capability-based chapter sequence is the main guide.

## Build

Use Python 3 with the pinned build dependencies. The website itself has no
runtime packages, third-party scripts, analytics, or remote assets.

~~~sh
python3 -m venv .book-venv
.book-venv/bin/pip install -r site/compact-journey/requirements.txt
.book-venv/bin/python site/compact-journey/build.py
python3 site/compact-journey/check.py
python3 -m http.server 4187 --directory docs
~~~

Open http://localhost:4187/book/. Static HTML chapters work with JavaScript
disabled. JavaScript adds search, filtering, theme selection, progress and
the pipeline controls. Storage failures leave reading available.

## Source and publication

- chapters/ contains the book text.
- chapters.json supplies reading order, capability groupings and technical-reference labels.
- commits.json is the curated Git catalogue of the backend journey.
- sources/edition.json records source heads and classification rules.
- sources/ retains public documentation and qualification summaries.
- assets/ contains the local stylesheet and interactions.
- build.py renders the fixed inputs into docs/book/.

The existing repository Pages source is rust-ast-docs, directory /docs.
A fast-forward documentation publication builds the site through that
existing configuration. No compiler workflow, proof run, new Actions
workflow, or Pages-setting change is needed.

The older site/rust-ast-storybook/package.py historically accepts only its
own root files. Use the new book builder to update docs/book/. Preserve
the complete book/ directory when packaging the earlier overview.

## Illustrations

Editable Mermaid sources live in diagrams/. The book and print view use
local, pre-rendered PNGs. Each chapter includes a text explanation and
links to the diagrams' Mermaid source. Ordinary builds reuse the
checked-in images.

After editing a diagram, use Mermaid CLI 11.12.0 to regenerate the images
and rebuild the book.

~~~sh
sh site/compact-journey/render-diagrams.sh /path/to/mmdc
~~~

## Refresh an edition

A refresh is an explicit editorial operation, separate from a normal build.
Update the named source heads and cutoff in freeze.py, then provide the
current source checkout, checkpoint and frozen public PR/issue metadata.

~~~sh
python3 site/compact-journey/freeze.py \
  --source-dir /path/to/compatibility/checkout \
  --checkpoint /path/to/resume.json \
  --pr-metadata /path/to/public-prs.tsv \
  --issue-metadata /path/to/public-issues.json
~~~

PR TSV fields are number, title, state, merge timestamp and head SHA.
Issue metadata is the public gh issue list JSON result. The published
book contains summaries and hashes, with workstation paths removed.

Review narrative claims against the original receipts before building.
Phase assignment follows the earliest selected head that contains a commit.
Topics are inferred from subjects and serve as navigation. The catalogue
retains both author and committer dates. Local-at-cutoff records have no
invented GitHub commit link.

## Historical scope

The edition identifies the merged Ledger 9 foundation, subsequent local arm64
qualification and remaining native Linux/full-corpus/two-contract work.
The technical reference describes the Ledger 8 architecture at named revisions;
the narrative explains subsequent checked-effect integration and Ledger 9 changes.
Complete upstream compatibility remains unqualified at this edition’s cutoff.

Chapters explain the problem, design choice, implementation result and validation
boundary. Internal planning labels are not chapter names or package versions.
Dates remain in provenance and the commit catalogue. Original Git subjects and
frozen source reports remain unchanged. See sources/documentation-audit.md
for the knowledge-base reconciliation.

An evidence refresh preserves authored chapter prose, titles and URLs. Reconcile
new facts editorially; do not regenerate reference chapters from older Markdown.

Content and source remain under the repository's Apache-2.0 license.

The downloadable ZIP fixes file metadata. Its compressed bytes are reproducible within the same Python/zlib environment and can differ across zlib versions.

## Commit references

The build replaces prose revision hashes with the corresponding commit
subjects. Published revisions link to GitHub. Local revisions have a
static book entry with their provenance. Use **Details** to open the
message, iteration context and full identity in a side panel.

When updating the edition or adding a reference, run
python3 site/compact-journey/revisions.py from the repository checkout
before rebuilding. It reads the edition's Git objects and freezes the
metadata in sources/revisions.json. Ordinary builds use that snapshot.

## Reading themes

Clear is the light theme, using a white page, Inter body text, and crisp
code panels. Dusk is the dark theme, using muted slate, PT Serif body
text, and softer code colors. Both follow the owner's selected local
reading previews. The theme control names its destination, and the
reader's choice is remembered. New readers start with Clear.

Fonts and licenses are bundled locally. The complete print view uses
a light page with dark text.

## Syntax highlighting

Compact, Rust, and TypeScript code fences are highlighted during the
Python build. Rust and TypeScript use pinned Pygments lexers. The Compact
lexer reuses keyword and built-in type patterns from the repository's
Compact editor grammar. Highlighting verifies that the complete token
stream preserves the example text. Plain-text plans retain their original
presentation.

Clear, Dusk and print colors are defined in assets/syntax.css.
The published HTML contains the token markup. It displays without a
client-side highlighting library or network request.

Run the focused checks with the same build environment.

~~~sh
python3 -m unittest discover -s site/compact-journey -p test_highlighting.py
~~~

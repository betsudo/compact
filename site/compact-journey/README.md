# From source to proof

A pre-rendered interactive book about Compact's journey from the Ledger 8
TypeScript reference path to a Ledger 9 Rust AST backend.

The November 13, 2026 edition contains 23 chapters, a searchable commit
catalogue, 37 curated iteration checkpoints, a pipeline explorer, local reading progress, light/dark themes,
and a complete print view. The original October storybook stays available
at the Pages root.

## Build

Use Python 3 with the exact build dependency. The website itself has no
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
- chapters.json supplies reading order, phases and historical-atlas labels.
- commits.json is the deduplicated frozen Git catalogue.
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

The merged Ledger 9 foundation, subsequent local arm64 qualification, and
remaining native Linux/full-corpus/two-contract work are explicitly dated.
The atlas preserves the October text. The current chapters explain later
progress and corrections. The milestone remains incomplete at this edition.

Content and source remain under the repository's Apache-2.0 license.

The downloadable ZIP fixes file metadata. Its compressed bytes are reproducible within the same Python/zlib environment and can differ across zlib versions.

---
format: aep.planning-md/1
id: story:authored-site-link-resolution
kind: story
status: implemented
title: Resolve authored documentation links in generated sites
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/Cargo.toml
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/site.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/authored_site.rs
- confidence: cited
  path: crates/generate/ess-gen/src/authored.rs
- confidence: cited
  path: docs/design/authored-site-publication.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
revision: 5
---
## Evidence

- crates/generate/ess-gen/src/authored.rs:29 parses authored Markdown into document blocks.
- crates/generate/ess-gen/src/html.rs:145 defines the site renderer and authored front-page support.
- crates/edge/ess-cli/src/main.rs:2277,2294 loads README and included pages.
- Reproduced on ESS 0.18.0: --include docs/context=<file> emits docs/context.html, while an authored link [Context](docs/context.md) remains href="docs/context.md". The site therefore points at a nonexistent file unless the adopter supplies an extra publishing adapter.

## Outcome

Authored system documentation, supporting chapters and contract downloads form a browsable site without adopter-specific HTML rewriting.

## Proposed Contract

Carry source-document identity and explicit included-page destinations into rendering. Resolve relative Markdown links against the source document, then map included documents to their published page IDs. Cover the front-page README, nested include IDs, README directory indexes, fragments and copied/downloadable contracts. Preserve external URLs, query strings and fragments. Do not turn arbitrary filesystem paths into public assets or widen the existing path-containment boundary.

Unpublished local targets must produce a visible source-located diagnostic under an explicit strict policy, rather than silently creating a broken link or copying private sibling files. Use the parsed Markdown/document representation, not regex substitution across generated HTML. Preserve deterministic output and current page-ID collision checks.

## Acceptance

A nested multi-page authored fixture has no broken local links after ESS site generation, external/download links remain correct, and missing or escaping local targets are diagnosed without publishing undeclared files.

## Scope

- Cited: crates/generate/ess-gen/src/authored.rs, html.rs, document model and CLI include loading.
- Inferred: declared asset/destination resolver and integration fixtures.
- No new site shell, live filesystem crawler, public deployment mechanism or changes to model semantics.



## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 5 and status implemented. Source artifact SHA-256: e423eeb365dc2c1aa8ddb06fc48d4c3fa48d88a7fc8b04281b8a97a843a07f37. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.

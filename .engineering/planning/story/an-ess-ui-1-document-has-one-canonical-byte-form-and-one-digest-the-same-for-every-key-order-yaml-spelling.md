---
format: aep.planning-md/3
id: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
kind: story
status: active
title: An ess-ui/1 document has one canonical byte form and one digest, the same for every key order, YAML spelling, anchor, comment, short form and filled default written out or left out
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
- depends_on: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:13Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:14Z", actor: "human:timo", revision: 5}
---
## Need

A program that generates `ess-ui/1` documents by different routes needs to know when two routes
produced one document. It also keys what it computed per document by the document's identity.
ESS defines no canonical form or digest of an `ess-ui/1` document.

- No UI crate computes a digest. `git grep -n "digest\|sha2" -- crates/ui/*/src crates/ui/*/Cargo.toml`
  finds nothing.
- The digest table of the formats reference (`website/docs/reference/formats.md:20-31`) has no UI
  row.
- `ess ui load` prints one summary line (`crates/ui/ess-ui/src/lib.rs:152-163`,
  `crates/edge/ess-cli/src/ui.rs:111-112`), not a resolved form. The report says it "prints the
  resolved form". That differs: it prints only the counts. `g03-resolved-json` adds the resolved
  form.
- ESS already publishes one scheme for documents a reader must be able to recompute: a bare
  SHA-256 of compact, key-sorted JSON with no trailing newline.
  - `InfraIR` uses it (`formats.md:29`, `crates/infra/infra-compiler/src/ir.rs:888-897`), and so
    does `InfraSpec` (`formats.md:30`).
  - The compiled `EssIr` digest hashes struct field order instead (`formats.md:24`,
    `crates/specify/ess-compiler/src/ir.rs:3863-3890`). A reader that parses the JSON cannot
    recompute it, and `infra-compiler/src/ir.rs:889-892` names exactly that as the reason for
    sorting keys.

The requester's words (report item 2): "A normative canonical byte form and digest of an `ess-ui/1`
document and of a UI state." The UI state half is `g01-state-document`.

## Outcome

- **The canonical form** of a document is `Loaded::tree` (`g03-loader-fills-defaults`), written as
  compact JSON. `g03-resolved-json` prints the same tree as its `document` member.
  - Object keys are sorted by Unicode code point. Arrays keep their order.
  - Strings are escaped as `serde_json` escapes them: `"`, `\` and control characters only, with
    other text kept as UTF-8.
  - A number with an integral value is written as a decimal integer. Any other number is written
    in its shortest round-trip ECMAScript form, which `ryu-js` already produces in `ess-ui`
    (`Cargo.toml`, `filter.rs:146`). So `1.0` and `1` are one number.
  - There is no whitespace and no trailing newline.
- **The digest** is the bare 64-character lowercase SHA-256 of those bytes.
- **The name of the rule** is "the `InfraIR` scheme with the ECMAScript number rule". The two
  differ only in numbers: `InfraIR` writes them with `serde_json::to_vec` (`ir.rs:893-896`), so
  `1.0` stays `1.0`. The digest-table row, which `g03-resolved-json` writes, says so.
- **The writer.** `ess_ui::canonical_json(&serde_json::Value) -> String` is the one writer in the
  UI area. `Loaded::json(&self) -> serde_json::Value` converts the tree. `Loaded::digest(&self)
  -> String` hashes it. `g01-state-document` reuses `canonical_json` for the state digest.
- **What the form ignores** (each is proved by acceptance 1): key order, YAML spelling (quoting,
  flow or block style, folded scalars), comments, anchors and aliases (the YAML reader expands
  them; `<<` is an ordinary key, `crates/specify/ess-yaml/src/lib.rs:9`), short forms, the
  spelling `1.0` against `1`, and a default written out against left out.
  - The default clause holds for every key that some fill unit writes. All of them have landed
    before this unit (round 5).
  - It does not hold for the keys that stay unfilled: `list.unique` inside `types`, `Degrades`,
    `form.fields` without a model, and a `store` or `cache` that no profile resolves.
- **What the form keeps:**
  - list order (the schema makes order meaningful only in lists,
    `schemas/ui/ess-ui.schema.yaml:9-12`);
  - every value;
  - a page's `kind`, which the merged tree keeps (`g03-resolved-json`), so a page merged over a
    kind and the same page written out without `kind` are two forms;
  - author text and traceability keys (P0-D7 A).
- **The model.** A document loaded with a model can hold model-filled defaults
  (`g03-model-defaults`), so its digest can differ from the same document loaded without one.
  `ess-ui-resolved/1`'s `model` member, which `g03-resolved-json` owns, says which case it is.

## Acceptance

New test file `crates/ui/ess-ui/tests/canonical.rs`. Each test is expected under 2 s and is timed;
none may run over 60 s, and the crate's test step stays within 120 s. The file joins the crate's
single test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it
is a new file, which that story moves.

1. `spellings_of_one_document_share_a_digest`. Red today: `canonical_json` and `Loaded::digest` do
   not exist. A base document and seven respellings give byte-equal canonical forms and equal
   digests. The respellings:
   - keys reordered;
   - block style against flow style;
   - comments added;
   - an anchor and an alias for a repeated column list;
   - a field written as a bare name against `{field: x}`;
   - a default written out (`style: table`) against left out;
   - a state's `default: 1.0` against `default: 1` (a `json` value).
2. `a_change_of_meaning_moves_the_digest`. Two sections swapped, a title changed and a node
   removed each give a different digest.
3. `the_canonical_form_is_a_fixed_point`. For every document of the UI corpus, loaded with the
   model it needs if any, the canonical bytes loaded again with `load_resolved` (JSON is YAML)
   give the same canonical bytes.
4. `the_digest_is_recomputable`.
   - The canonical bytes and the digest of `examples/partner-portal/ui.yaml` are pinned in
     `crates/ui/ess-ui/tests/golden/partner-portal.canonical.json` and
     `crates/ui/ess-ui/tests/golden/partner-portal.canonical.digest`. This unit is the files' only
     owner.
   - The test recomputes the digest from the golden bytes with `sha2` alone.
   - It re-sorts the golden file's keys with `serde_json::Value` and finds the bytes unchanged.
   - The two files are written by `ESS_UI_GOLDEN=write cargo test -p ess-ui --test canonical`.
     A later story that moves them names that command in its Scope.
5. `the_stable_surface_compiles` (`crates/ui/ess-ui-test/tests/session.rs`, from
   `g01-session-actions`) adds `canonical_json`, `Loaded::json` and `Loaded::digest`.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui --locked` and `cargo test -p ess-ui-test --test session --locked`;
- `task test-xtask` (the dependency boundary test `crates/edge/ess-xtask/tests/layout.rs`).

Also:
- `formats.md` is not changed here. The digest-table row names `ess-ui-resolved/1`, and
  `cargo xtask docs` refuses a format family that `FORMAT_RELEASES` does not track yet
  (`docs.rs:530-533`), so `g03-resolved-json` writes the row with the format, in the next round
  and the same release.
- `crates/ui/ess-ui/Cargo.toml` adds `sha2` (`workspace = true`, `Cargo.toml:109`). It is an
  outside crate, so the rule "no dependency on the rest except `ess-yaml`" holds.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 2.

1. **Need.** One identity per document, independent of how it was written. Reproduction: write
   `examples/partner-portal/ui.yaml` with its keys reordered. ESS offers nothing that says the two
   files are one document, and a byte hash of the files differs.
2. **Class.** It is a *gap*.
3. **Already expressible?** No. A program can hash its own serialisation of `ess ui load`'s
   output, but that output is a count. Even with the resolved form, two programs need one
   normative byte rule to agree.
4. **Fit.**
   - It reuses the published `InfraIR` scheme with one stated difference, numbers. The `EssIr`
     scheme is not reused, because a reader cannot recompute it.
   - It is defined over the filled tree, so "a default written or left out" is settled by the
     fill units and not by a second normaliser.
   - No authored key and no `ess-ui/1` version.
5. **Second adopter.** A documentation site caches the screenshots of a generated UI by document
   digest and regenerates only when the digest moves.
6. **Cost.**
   - One digest domain added to the table.
   - A second implementation of a key-sorted JSON hash: `ess-ui` may not depend on
     `infra-compiler`, whose `digest_of_canonical` (`ir.rs:925-940`) is the other one. The two
     rules differ by design in numbers, so no drift test between them is written.
   - A change of a filled default changes digests, as it changes resolved bytes. That is a change
     of meaning of `ess-ui/1` in any case.
7. **Considered.**
   - (a) A hash of the authored bytes. Rejected: it is sensitive to everything the need excludes.
   - (b) RFC 8785 (JCS). It gives nearly the same bytes, but ESS has no JCS writer, and its UTF-16
     key order differs from the code-point order used elsewhere in ESS.
   - (c) Chosen: the `InfraIR` scheme, with the ECMAScript number rule, over the filled tree.

## Decisions

- **P0-D6: Which canonical byte rule and digest?**
  - (A) The `InfraIR` scheme with the ECMAScript number rule: compact, key-sorted JSON, bare
    SHA-256, no trailing newline.
  - (B) RFC 8785 with a `jcs-sha256:` prefix.
  - (C) The `EssIr` style, serialising in struct order. A reader cannot recompute it.

  **Recommended: (A).**
- **P0-D7: Does the digest cover author-only text and traceability keys** (`doc`, `summary`,
  `source`, `endpoint`, `roles_today`, `transport_today`, `unmapped`; schema `traceability`,
  `:1743-1748`)?
  - (A) Yes. The digest is the document's identity.
  - (B) No. It covers what renders only, and the schema would need a machine-readable "author
    only" mark it does not have today.
  - (C) Both, as two digests.

  **Recommended: (A).** A program that wants a behaviour-only identity strips those keys from the
  printed `document` and hashes the rest by the same rule.

## Scope

- new: `crates/ui/ess-ui/src/canonical.rs`: `canonical_json`, `Loaded::json`, `Loaded::digest`,
  about 80 lines.
- `crates/ui/ess-ui/src/lib.rs`: `mod canonical` and the export.
- `crates/ui/ess-ui/Cargo.toml` (`sha2`), and `Cargo.lock` (the dependency line of `ess-ui`).
- new: `crates/ui/ess-ui/tests/canonical.rs`,
  `crates/ui/ess-ui/tests/golden/partner-portal.canonical.json`,
  `crates/ui/ess-ui/tests/golden/partner-portal.canonical.digest`.
- `crates/ui/ess-ui-test/tests/session.rs`: three names added to `the_stable_surface_compiles`.
- `CHANGELOG.md`.
- Not changed: `crates/edge/ess-cli/src/ui.rs` and `website/docs/reference/formats.md`.
  `g03-resolved-json` prints `digest` and writes its digest-table row.
- Size: S to M, about 260 changed lines: code about 90, tests about 170.

## Depends on

- `g03-loader-fills-defaults`: `Loaded`.
- `g03-defaults-that-move-behaviour`, the last fill unit (round 4). The golden bytes must not move
  after this unit.
- `g01-session-actions` (round 3): `the_stable_surface_compiles` in `ess-ui-test/tests/session.rs`.

Round 5, alone. `g03-resolved-json` (round 6) and `g01-state-document` (round 7) depend on it.

---
format: aep.planning-md/3
id: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
kind: story
status: draft
title: An ess-ui/1 document has one canonical byte form and one digest, the same for every key order, YAML spelling, anchor, comment, short form and default written out or left out
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
revision: 1
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
  - It is used by `InfraIR` (`formats.md:29`, `crates/infra/infra-compiler/src/ir.rs:888-897`) and
    by `InfraSpec` (`formats.md:30`).
  - The compiled `EssIr` digest hashes struct field order instead (`formats.md:24`,
    `crates/specify/ess-compiler/src/ir.rs:3863-3890`). A reader that parses the JSON cannot
    recompute it, and `infra-compiler/src/ir.rs:889-892` names exactly that as the reason for
    sorting keys.

The requester's words (report item 2): "A normative canonical byte form and digest of an `ess-ui/1`
document and of a UI state." The UI state half is `g01-state-document`.

## Outcome

- The canonical form of a document is the `document` member of `ess-ui-resolved/1`
  (`g03-resolved-json`), written as compact JSON.
  - That member is the loaded tree after every short form is expanded, page kinds and `same_as`
    are merged, widgets are instantiated, tone and label maps are resolved, and every default is
    filled (`g03-loader-fills-defaults`).
  - Object keys are sorted by Unicode code point. Arrays keep their order.
  - Strings are escaped as `serde_json` escapes them: `"`, `\` and control characters only, with
    other text kept as UTF-8.
  - An integer is written in decimal. Any other number is written in its shortest round-trip
    ECMAScript form, which `ryu-js` already produces in `ess-ui` (`Cargo.toml`, `filter.rs:146`).
  - There is no whitespace and no trailing newline.
- The digest is the bare 64-character lowercase SHA-256 of those bytes.
- `ess_ui::canonical_json(&serde_json::Value) -> String` is the one writer in the UI area.
  `g01-state-document` reuses it for the state digest. `Resolved::digest()` applies it.
  `ess ui load --format json` prints `digest` beside `document`.
- The form does not depend on key order, YAML spelling (quoting, flow or block style, folded
  scalars), comments, anchors and aliases (the YAML reader expands them; `<<` is an ordinary key,
  `crates/specify/ess-yaml/src/lib.rs:9`), short forms, page kinds, widgets, `same_as`, map
  references, a default written out or left out, or the spelling `1.0` against `1`.
- The form depends on list order (the schema makes order meaningful only in lists,
  `schemas/ui/ess-ui.schema.yaml:9-12`), on every value, and under P0-D7 (A) on author text and
  traceability keys.
- A document loaded with `--model` can hold model-filled defaults (`g03-model-defaults`), so its
  digest can differ from the same document loaded without a model. `ess-ui-resolved/1` says which
  case it is (`model: true|false`).

## Acceptance

New test file `crates/ui/ess-ui/tests/canonical.rs`. Each test is expected under 2 s and is timed.
None may run over 60 s, and the file's step stays within 120 s.

1. `spellings_of_one_document_share_a_digest`. Red today: `canonical_json` and `Resolved::digest`
   do not exist.
   - A base document and seven respellings give byte-equal canonical forms and equal digests.
   - The respellings:
     - keys reordered;
     - block style against flow style;
     - comments added;
     - an anchor and an alias for a repeated column list;
     - a field written as a bare name against `{field: x}`;
     - a default written out (`style: table`) against left out;
     - a state's `default: 1.0` against `default: 1` (a `json` value).
2. `a_change_of_meaning_moves_the_digest`. Two sections swapped, a title changed and a node
   removed each give a different digest.
3. `the_canonical_form_is_a_fixed_point`. The canonical bytes, loaded again with
   `ess_ui::load_str` (JSON is YAML), give the same canonical bytes. This holds for every document
   under `examples/` and every document of the `ess-ui` test corpus.
4. `the_digest_is_recomputable`.
   - The canonical bytes and the digest of `examples/partner-portal/ui.yaml` are pinned in
     `crates/ui/ess-ui/tests/golden/partner-portal.canonical.json` and its `.digest` beside it.
   - The test recomputes the digest from the golden bytes with `sha2` alone.
   - It also re-sorts the golden file's keys with `serde_json::Value` and checks the bytes are
     unchanged.
5. `ess ui load --path examples/partner-portal/ui.yaml --format json` prints `digest` equal to the
   pinned digest (in `crates/edge/ess-cli/tests/`, beside `g03-resolved-json`'s test).

Also:

- `formats.md` "Which digest is this?" gets a row: "UI document: `ess-ui-resolved/1` `digest`.
  Bare SHA-256 of compact, key-sorted JSON of `document`, no trailing newline; not a hash of the
  authored YAML." The `ess-ui-resolved/1` row names `digest` and `model`.
- `crates/ui/ess-ui/Cargo.toml` adds `sha2` (`workspace = true`, `Cargo.toml:109`). It is an
  outside crate, so the rule "no dependency on the rest except `ess-yaml`" holds.
  `crates/edge/ess-xtask/tests/layout.rs` passes.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 2.

1. **Need.** One identity per document, independent of how it was written. Reproduction: write
   `examples/partner-portal/ui.yaml` with its keys reordered. ESS offers nothing that says the two
   files are one document. A byte hash of the files differs.
2. **Class.** This is a *gap*.
3. **Already expressible?** No. A program can hash its own serialisation of `ess ui load`'s
   output, but that output is a count. Even after `g03-resolved-json`, two programs would need one
   normative byte rule to agree.
4. **Fit.**
   - It reuses the published `InfraIR`/`InfraSpec` scheme rather than a new one. The `EssIr` scheme
     is not reused, because a reader cannot recompute it.
   - It is defined over the resolved document, so "defaults written or left out" is settled by
     `g03-loader-fills-defaults` and not by a second normaliser.
   - There is no authored key and no `ess-ui/1` version.
5. **Second adopter.** A documentation site caches the screenshots of a generated UI by document
   digest and regenerates only when the digest moves.
6. **Cost.**
   - One digest domain added to the table.
   - The canonical writer is a second implementation of the key-sorted JSON hash. `ess-ui` may not
     depend on `infra-compiler`, whose `digest_of_canonical` (`ir.rs:925-940`) is the other one.
     The golden vector test pins the bytes so that the two cannot drift silently.
   - A change of a filled default changes digests, as it changes resolved bytes. That is a change
     of meaning of `ess-ui/1` in any case.
7. **Considered.**
   - (a) A hash of the authored bytes. Rejected: it is sensitive to everything the need excludes.
   - (b) RFC 8785 (JCS). It is nearly the same bytes, but ESS has no JCS writer, and its UTF-16 key
     order differs from the code point order used elsewhere in ESS.
   - (c) Chosen: the `InfraIR` scheme over the resolved document.

## Decisions

- **P0-D6: Which canonical byte rule and digest?**
  - (A) The `InfraIR` scheme: compact, key-sorted JSON, bare SHA-256, no trailing newline.
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

- new: `crates/ui/ess-ui/src/canonical.rs` (`canonical_json`, about 60 lines)
- cited: `crates/ui/ess-ui/src/lib.rs` (`Resolved::digest`, from `g03-resolved-json`),
  `crates/ui/ess-ui/Cargo.toml` (`sha2`)
- cited: `crates/edge/ess-cli/src/ui.rs` (print `digest` and `model` in the JSON output)
- new: `crates/ui/ess-ui/tests/canonical.rs`, `crates/ui/ess-ui/tests/golden/`
- `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: S to M, about 260 changed lines (code about 90, tests about 170).

## Depends on

- `g03-resolved-json` (the resolved tree and `ess ui load --format json`)
- `g03-loader-fills-defaults` (defaults written or left out give one form)

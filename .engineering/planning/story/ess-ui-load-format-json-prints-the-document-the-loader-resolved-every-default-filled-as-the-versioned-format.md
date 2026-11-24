---
format: aep.planning-md/3
id: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
kind: story
status: active
title: ess ui load --format json prints the document the loader resolved, every default filled, as the versioned format ess-ui-resolved/1
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
- depends_on: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
- depends_on: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
- depends_on: story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once
- depends_on: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:16Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:17Z", actor: "human:timo", revision: 4}
---
## Need

An application that writes its own generator from an `ess-ui/1` document gets nothing it can read
from `ess ui load`. The command prints one summary line, `ess-ui/1: <n> pages, <m> addressed nodes`
(`crates/ui/ess-ui/src/lib.rs:152-163`, `crates/edge/ess-cli/src/ui.rs:105-115`). To get what the
stock generators start from, such a generator has to re-implement:

- the loader's steps: short forms, page kinds and `same_as`, widgets, and tone and label maps
  (`lib.rs:4-14`, about 1,870 lines in `expand.rs`);
- canonical node paths (`path.rs:108-110`, `Document::nodes`);
- every default of the schema, which the stock renderers hold in their own code, and on which they
  disagree (the table in `g03-loader-fills-defaults`).

The stock generators do not parse YAML. They take the typed `Document` from `ess_ui::load_path` or
`load_path_with` (`crates/ui/ess-ui-react/src/lib.rs:218`, `:263-264`, `:286`;
`crates/ui/ess-ui-test/src/lib.rs:201`, `:354`). `Document` is Deserialize-only (`model.rs`, no
`Serialize` derive), and the expanded tree is private to `load` (`lib.rs:111-126`), so nothing can
print it.

Reproduction: `ess ui load --path examples/partner-portal/ui.yaml` prints only the summary line,
and `ess ui load` has no `--format` (`ui.rs:31-41`).

The requester's words: "`ess ui load --format json` prints the resolved document: every addressed
node with its node path, every default filled in, every model reference resolved to its qualified
name and type, and a format id and schema (for example `ess-ui-resolved/1`) published with the
reference so a generator can pin it. `ess generate ui --target react` reads the same form, so a
stock and an own generator start from identical input."

The owner, 2026-11-24: the loader fills every default.

## Outcome

- **Published once, complete.** This unit lands after every fill unit:
  `g03-loader-fills-defaults`, `g03-actor-from-session`, `g03-model-defaults`,
  `g03-placement-in-loader` and `g03-defaults-that-move-behaviour`. `g03-resolved-binding` lands in
  the next round. Both are in the same wave as `g02-document-digest`, `g01-state-document` and
  `g01-state-cli`, so one release publishes `--format json`, the formats row, the
  `FORMAT_RELEASES` entry and the golden files. Digests are stable from that release on.
- **The document.** `ess ui load --format json` prints an `ess-ui-resolved/1` document
  `{format, model, digest, document, nodes}`:
  - `format`: `"ess-ui-resolved/1"`;
  - `model`: `true` when loaded with `--model`;
  - `digest`: `Loaded::digest` (`g02-document-digest`);
  - `document`: `Loaded::json`, the tree after expansion, merging, instantiation and the fill, in
    the keys of `ess-ui/1`;
  - `nodes`: every addressed node's canonical path and construct, in `Document::nodes` order;
  - `binding`: added by `g03-resolved-binding` under `--model`, in the same release. This unit's
    formats row already names it as present only with `--model`.
- **The printed bytes** are pretty JSON with keys sorted, ending in one newline. The digest is
  computed over the canonical form, not over these bytes.
- **Versioned and recorded.** The format is Serialize-only and has a row in the formats
  reference. A live golden file pins its bytes for an example. A frozen golden file of the first
  release holds the old reader's input. A round-trip test proves that the printed document reads
  back to the `Document` the stock generators start from.
- **The renderers' own fallbacks.** Once the loader fills a key, both renderers read the filled
  value. A fallback that agreed with the schema stays as dead code until the draft follow-up
  `g03-no-renderer-fallback`, which is not part of P0, removes it. Where a fallback disagreed, the
  fill units moved the behaviour or corrected the schema, each named in its own story.

## Acceptance

New test files `crates/ui/ess-ui/tests/resolved.rs` and `crates/edge/ess-cli/tests/ui_load_json.rs`.
Each test is expected under 5 s and is timed; none may run over 60 s, and each crate's test step
stays within 120 s. Each file joins its crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

1. `ess_ui_load_prints_the_resolved_document`. Red today: `--format` is an unknown argument.
   `ess ui load --path examples/partner-portal/ui.yaml --format json` exits 0 and prints one JSON
   object. It holds exactly the members the formats row lists for a run without `--model`, with
   `format` equal to `"ess-ui-resolved/1"` and `model` equal to `false`.
2. `text_output_is_unchanged`. `--format text`, the default, prints the summary line it prints
   today, byte for byte. A refusal is the one `ess ui load` prints today, on stderr, with exit 1, in
   both formats.
3. `the_document_merges_every_kind_and_widget`. In `document`, every page kind, `same_as` overlay
   and widget instance of the partner portal is merged, and every short form is expanded.
4. `the_document_holds_only_schema_keys_and_the_kept_list`. At each construct, the set of keys in
   `document` is a subset of the schema's keys for that construct plus the test's pinned `KEPT`
   list. `KEPT` holds the keys a merge consumes but the reload needs, starting with `kind`, which
   `Page` requires.
5. `consumed_keys_are_named`. The test pins `CONSUMED`, the keys a merge consumes and the printed
   document leaves out, and asserts that no key of `CONSUMED` appears in `document`. The
   implementer fills both lists from the loader and names each key's reason.
6. `nodes_match_the_loader`. `nodes` lists every node `Document::nodes` yields, in that order, as
   `{path, construct}`, where `construct` is the `NodeRef` variant in snake case. Its length equals
   the summary line's addressed-node count.
7. `a_states_block_of_defaults_reads_back`. A page whose `states` holds only defaults is printed
   and read back by `load_resolved` without a refusal.
8. `the_printed_document_reads_back`. For every document of the UI corpus, `load_resolved` reads
   the printed `document` (JSON is YAML) back to a `Document` equal to the one it was printed
   from.
   - A document that names a model enum in a choice's `options` is printed and read back with the
     same model. `document` keeps such `options` as the enum's name, because the variants live in
     `Document::model_enums`, outside the tree (`lib.rs:121`).
   - Printing the read-back document gives the same bytes.
9. `the_bytes_are_pinned`. The printed bytes of `examples/partner-portal/ui.yaml` equal the live
   golden `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json`. Printing twice gives
   equal bytes. The file is written by
   `cargo run -q -p ess-cli -- ui load --path examples/partner-portal/ui.yaml --format json > crates/edge/ess-cli/tests/golden/partner-portal.resolved.json`,
   and a later story that moves it names that command in its Scope.
10. `an_old_reader_still_reads`. The frozen golden
    `crates/edge/ess-cli/tests/golden/partner-portal.resolved.first-release.json` is a byte copy of
    the live golden as this unit writes it, and no later story edits it. The test checks that
    every key the frozen file holds is still printed, with the same meaning, for the same input.
    An optional key added later is left out when it is empty, as `ess-ui-check/1` does
    (`website/docs/reference/formats.md:219`).

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-cli --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui --locked`;
- `cargo test -p ess-cli --test ui_load_json --locked`;
- `task test-xtask` and `task ci-lint`;
- `task projection-check`: its `cargo xtask docs` refuses a format family named under
  `website/docs` that `FORMAT_RELEASES` does not track (`docs.rs:530-533`, `:990-1012`), and its
  `cargo xtask cli-reference --check` compares `cli.md`.

Also:
- `website/docs/reference/formats.md` gets a row, "UI resolved document: `format: ess-ui-resolved/1`".
  It names:
  - the release that added the format, and that it is Serialize-only;
  - the members: `format`, `model`, `digest`, `document`, `nodes`, and `binding` only with
    `--model`;
  - what keeps the version and what changes it;
  - that `document` uses the keys of `ess-ui/1` and holds every default except `list.unique`
    inside `types`, `Degrades`, `form.fields` without `--model` (`model: false`), and a `store` or
    `cache` that no profile resolves;
  - the five kinds of content listed under "What the resolved document does not carry".
- The digest table of `formats.md` gets the row "UI document: `ess-ui-resolved/1` `digest`. Bare
  SHA-256 of compact, key-sorted JSON of `document`, numbers in ECMAScript form, no trailing
  newline; not a hash of the authored YAML." It lands here, with the format and its
  `FORMAT_RELEASES` entry, and not in `g02-document-digest`, because `cargo xtask docs` refuses a
  format family that `FORMAT_RELEASES` does not track yet.
- `crates/edge/ess-xtask/src/docs.rs`, `FORMAT_RELEASES` (`:396-399`), gets
  `("ess-ui-resolved", 1, Some("<release>"))`.
- The `ess ui load` entry of the CLI reference (`website/docs/reference/cli.md:1855-`) lists
  `--format`. `cli.md` is regenerated by `cargo xtask cli-reference`, never edited by hand.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Nothing under
`crates/ui`, `schemas/ui` or `crates/edge/ess-cli/src/ui.rs` changed since `fad42d26f`, where the
earlier draft (never filed in the store) was read. The request came from a downstream application on
2026-11-24, report item 3, its finding F3.

1. **Need.** A generator for another platform starts from the same resolved document as the stock
   generators, without re-implementing the loader or the defaults. The reproduction is above.
   - A second finding, from the requester's reply on 2026-11-24: an own generator re-implements
     every entry of the reference's Default column. The schema declares 75 `default:` entries.
   - **Evidence received 2026-11-24.** A downstream application's own generator, written against
     `ess` 0.98.0, has 1,807 lines. 1,296 of them re-implement ESS:

     | re-implemented | lines | after P0 |
     |---|---|---|
     | YAML reader | 493 | removed: the resolved document is JSON |
     | defaults and short forms | 144 | removed: both are expanded and filled in `document` |
     | page resolution (kinds, `same_as`, widgets) | 117 | removed: merged in `document` |
     | model binding | 195 | partly removed by `g03-resolved-binding`: names and routes. Types stay with the generator (P0-D11). |
     | live semantics | 134 | kept (see below) |
     | tokens | 91 | kept (see below) |
     | fixtures | 79 | kept (see below) |
     | expressions | 35 | kept (see below) |

     The requester estimates that `ess ui load --format json` with the bindings resolved removes
     about 960 lines. The three rows removed in full sum to 754. With the model binding row
     counted in full, the sum is 949. How much of those 195 lines is types was not established.
   - The requester also states that the ESS crates are not published (`Cargo.toml:61`,
     `publish = false`) and that the typed document cannot be serialised. So linking the crates is
     no alternative for them. This supports P0-D8 (A) and rejects alternative (a) below.
   - Consequences for the units:
     - Short forms are expanded as well as defaults filled.
     - Page resolution is in `document`.
     - The model binding is part of P0, not a follow-up.
2. **Class.** It is a *gap*. The resolved form exists only as Rust values. A Rust generator can
   link `ess-ui`; a generator in another language has no route. `ess ui check --format json`
   prints findings, not the tree (`formats.md:219`).
3. **Already expressible?** No. The route table is serialised into every generated React project
   as `src/binding.ts` (`crates/ui/ess-ui-react/src/assets.rs:286-296`), but reaching it means
   running the React generator and parsing TypeScript.
4. **Fit.**
   - No language grid applies (`docs/design/language-grids.md:15`, `:61-63`), and no authored key
     is added.
   - The document part is the loader's own expanded tree in `ess-ui/1`'s vocabulary: not a new
     shape and not a property bag (`AGENTS.md` § Boundaries).
   - `--format json` and the `ess-ui-<what>/1` naming follow `ess ui check --format json` and
     `ess ui test --format json` (`formats.md:219-221`).
   - Every target keeps reading the same in-memory values, and the round-trip test holds the
     printed form to them.
5. **Second adopter.**
   - A team generates a native mobile client from the same document.
   - A test-planning tool lists every addressed node path, so that its `ess-ui-test/1` steps name
     only nodes that exist.
6. **Cost.**
   - One new Serialize-only output format, `ess-ui-resolved/1`. It is a persisted interchange
     format, because a downstream generator pins it. The duty is two golden files, an old-reader
     test and a row in the formats reference.
   - A key that `ess-ui/1` adds later appears in `document` only when the document uses it or the
     schema gives it a default, so the format stays `/1`. A renamed or removed key needs `/2`.
   - A changed schema default changes resolved bytes and digests, not the shape. That is a change
     of meaning of `ess-ui/1`, and it moves the live golden file visibly.
   - One flag on `ess ui load`. No diagnostic, no migration.
   - For contrast, the compiled `EssIr` is deliberately unversioned, with no persisted reader
     (`formats.md:204`). That fits an output nobody pins. This output exists to be pinned.
7. **Considered.**
   - (a) Change nothing: a Rust generator links `ess-ui`. Rejected: other languages get no route.
     The crates are unpublished and `Document` cannot be serialised, which the requester's own
     generator confirms.
   - (b) Derive `Serialize` on every type in `model.rs` and print `Document`. Rejected: about
     3,700 lines of types would need symmetric serde attributes, and the schema would get a second
     source of key spelling.
   - (c) A new purpose-built shape. Rejected: new vocabulary for one request.
   - (d) Chosen: the expanded and filled tree in `ess-ui/1` keys, plus the node index.
   - From the request:
     - Taken: the flag, "every addressed node with its node path", "every default filled in", the
       format id and the pin.
     - Changed: "resolved to its … type" becomes the route table (`g03-resolved-binding`, P0-D11).
       "`ess generate ui` reads the same form" becomes a round-trip test (P0-D10).
     - Changed: "a … schema published with the reference" becomes the formats row. `document` is
       described by `schemas/ui/ess-ui.schema.yaml`, which is already published, and the envelope
       by that row. No output format of the UI area has a schema file today.

## Decisions

**Accept, redesigned.** The defaults question of the earlier draft (its D3) was settled by the
owner as "the loader fills every default". The open questions:

- **P0-D8 (the draft's D1): Is the printed form a versioned format?**
  - (A) Yes: `ess-ui-resolved/1`, with a formats row, two golden files and an old-reader test.
  - (B) No: unversioned output, like `EssIr`.
  - (C) Decline, with the idiom "link `ess-ui`".

  **Recommended: (A).**
- **P0-D9 (the draft's D2): What is the document part?**
  - (A) The loader's expanded and filled tree, which `ess ui load` reads back.
  - (B) `Serialize` on `Document`.
  - (C) A new shape.

  **Recommended: (A).**
- **P0-D10 (the draft's D4): Does `ess generate ui --target react` read the printed JSON?**
  - (A) No. It keeps the in-memory `Document`, and the round-trip test proves the two equal.
  - (B) Yes, through a JSON hop.

  **Recommended: (A).**

### What the resolved document does not carry

An own generator still implements the following. The `ess-ui-resolved/1` row in the formats
reference says so.

- **Live semantics (134 lines in the evidence).** The document carries every `live` and `channel`
  declaration, filled. What an effect does at run time (where an insert goes, coalescing, paging
  away, resume) is behaviour, not data. A later story could cover it: the `ess-ui-test/1` corpus
  published as a conformance suite that an own generator runs against its output. That needs a
  harness for a third-party renderer, so it gets its own design.
- **Fixtures (79 lines).** Fixture files and channel scripts are not part of the document. A later
  story could cover it: a resolved fixture index (each view's answers, `by_params`, `derived`,
  `fails`, and each script parsed), printed by the same command or a sibling, as a versioned
  format of its own.
- **Expressions (35 lines).** The document carries each binding expression as written text. A later
  story could print each one parsed, using the one grammar `ess-ui` already has (`filter.rs`,
  `token_grammar.rs`). Evaluating an expression stays the generator's own work.
- **Tokens (91 lines).** These are styling, which the owner has ruled out of scope: themes and
  templates belong to the downstream application. The built-in token table is published as data
  in the schema (`constructs.Tokens.builtins`), and an application that uses its own themes does
  not need it. No later story.
- **Keys filled but implemented by no stock renderer:** `Navigation.visibility`,
  `references.navigates`, `State.clear_on`, `Channel.scope`; in the terminal `Preload.policy`,
  `SectionStates.refreshing` and `collection.selection`; and in React `Live.when_paged_away`
  without a filter. Each is a defect outside P0. The value printed is the schema's meaning, not
  what a stock renderer does today.

Which keys the author wrote is not part of the document and not needed: a filled key and the same
key written out behave alike in every stock renderer (P0-D14 B, `g03-placement-in-loader`).

## Scope

- `crates/ui/ess-ui/src/path.rs`: `NodeRef`, a construct name per variant.
- `crates/edge/ess-cli/src/ui.rs`: `Load` 31-41 gains `--format`; `run` 105-115 prints the
  envelope.
- new:
  - `crates/ui/ess-ui/tests/resolved.rs`;
  - `crates/edge/ess-cli/tests/ui_load_json.rs`;
  - `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json`, the live golden;
  - `crates/edge/ess-cli/tests/golden/partner-portal.resolved.first-release.json`, the frozen
    golden.
- `website/docs/reference/formats.md` (the format row and the digest-table row),
  `website/docs/reference/cli.md` (generated), `crates/edge/ess-xtask/src/docs.rs`.
- `CHANGELOG.md`.
- Not changed: `crates/ui/ess-ui/src/lib.rs`, which already has `Loaded` and `Loaded::json`.
- Overlap: no open story covers a resolved UI form or `ess ui load` output.
  `story:ui-binding-contract` (implemented) made `Binding` serialise.
- Size: M, about 450 changed lines: code about 140, tests about 250, docs about 60.

## Depends on

- `g03-loader-fills-defaults`, `g03-actor-from-session`, `g03-model-defaults`,
  `g03-placement-in-loader` and `g03-defaults-that-move-behaviour`: every default the format
  promises is filled.
- `g02-document-digest`: `Loaded::digest` and its canonical form; this unit prints it and writes
  its digest-table row.

Round 6, alone. `g03-resolved-binding` (round 7) appends to `ui_load_json.rs` and edits `run` in
`ui.rs` after it.

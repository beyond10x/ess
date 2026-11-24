---
format: aep.planning-md/3
id: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
kind: story
status: draft
title: ess ui load --format json prints the document the loader resolved, every default filled, as the versioned format ess-ui-resolved/1
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
revision: 1
---
## Need

An application that writes its own generator from an `ess-ui/1` document gets nothing it can read
from `ess ui load`. The command prints one summary line, `ess-ui/1: <n> pages, <m> addressed nodes`
(`crates/ui/ess-ui/src/lib.rs:152-163`, `crates/edge/ess-cli/src/ui.rs:105-115`). To get what the
stock generators start from, such a generator has to re-implement:

- the loader's steps: short forms, page kinds and `same_as`, widgets, and tone and label maps
  (`lib.rs:4-14`, about 1,870 lines in `expand.rs`);
- canonical node paths (`path.rs:108-110`, `Document::nodes`);
- every default of the schema. The stock renderers hold most defaults in their own code, and
  disagree on eleven. The table is in `g03-loader-fills-defaults`.

The stock generators do not parse YAML. They take the typed `Document` from `ess_ui::load_path` or
`load_path_with` (`crates/ui/ess-ui-react/src/lib.rs:218`, `:263-264`, `:286`;
`crates/ui/ess-ui-test/src/lib.rs:201`, `:354`). `Document` is Deserialize-only (`model.rs`, no
`Serialize` derive), and the expanded tree is private to `load` (`lib.rs:111-126`), so nothing can
print it.

Reproduction: `ess ui load --path examples/partner-portal/ui.yaml` prints only the summary line.
`ess ui load` has no `--format` (`ui.rs:31-41`).

The requester's words: "`ess ui load --format json` prints the resolved document: every addressed
node with its node path, every default filled in, every model reference resolved to its qualified
name and type, and a format id and schema (for example `ess-ui-resolved/1`) published with the
reference so a generator can pin it. `ess generate ui --target react` reads the same form, so a
stock and an own generator start from identical input."

The owner, 2026-11-24: the loader fills every default.

## Outcome

`ess ui load --format json` prints an `ess-ui-resolved/1` document `{format, model, document,
nodes}`:

- `format`: `"ess-ui-resolved/1"`.
- `model`: `true` when the document was loaded with `--model`.
- `document`: the tree the loader deserialises in step 4, in the keys of `ess-ui/1`. It is printed
  after expansion, merging, instantiation and the default fill (`g03-loader-fills-defaults` and
  its follow-ups), with every default filled.
- `nodes`: every addressed node's canonical path and construct, in `Document::nodes` order.
- The `digest` of `document` is added by `g02-document-digest`. The route table under `--model` is
  added by `g03-resolved-binding`, which is part of P0 and ships in the same release (the evidence
  in the fit review counts the model binding at 195 lines).
- The format is versioned, Serialize-only and written in the formats reference. A golden file pins
  its bytes for an example, and a round-trip test proves that the printed document reads back to
  the very `Document` the stock generators start from.

**The renderers' own fallbacks.** Once the loader fills a key, both renderers read the filled value.

- Where a fallback agreed with the schema, it is now dead code. `g03-no-renderer-fallback` removes
  it.
- Where it disagreed, the behaviour changes to the schema's documented default, in
  `g03-defaults-that-move-behaviour`, `g03-actor-from-session`, `g03-terminal-follows-defaults`,
  `g03-react-follows-defaults` and `g03-placement-in-loader`.
- The eleven disagreements and their sides are listed in `g03-loader-fills-defaults`. Three schema
  texts are corrected rather than followed (P0-D12, P0-D13, P0-D16).

## Acceptance

New test files `crates/ui/ess-ui/tests/resolved.rs` and `crates/edge/ess-cli/tests/ui_load_json.rs`.
Each test is expected under 5 s and is timed. None may run over 60 s, and each file's step stays
within 120 s.

1. `ess_ui_load_prints_the_resolved_document`. Red today: `--format` is an unknown argument.
   `ess ui load --path examples/partner-portal/ui.yaml --format json` prints one JSON object
   `{format: "ess-ui-resolved/1", model: false, document, nodes}` and exits 0.
2. `text_output_is_unchanged`. `--format text`, the default, prints the summary line it prints
   today, byte for byte. A refusal is the one `ess ui load` prints today, on stderr, with exit 1, in
   both formats.
3. `the_document_holds_only_ess_ui_keys`.
   - `document` holds every page kind, `same_as` overlay and widget instance merged, every short
     form expanded, and every default the fill step writes.
   - No key exists in it that `schemas/ui/ess-ui.schema.yaml` does not declare at that construct.
     The walk of `g03-loader-fills-defaults` checks this.
   - A key the merge consumed is left out only if the reloaded document still loads. Otherwise it
     is kept: a page keeps `kind`, which `Page` requires.
4. `nodes_match_the_loader`. `nodes` lists every node `Document::nodes` yields, in that order, as
   `{path, construct}`, where `construct` is the `NodeRef` variant in snake case. Its length equals
   the summary line's addressed-node count.
5. `the_printed_document_reads_back`. For every committed document under `examples/` and every
   document of the `ess-ui` test corpus, `ess_ui::load_str` reads the printed `document` (JSON is
   YAML) back to a `Document` equal to the one it was printed from. Printing it again gives the
   same bytes.
6. `the_bytes_are_pinned`. The printed bytes of `examples/partner-portal/ui.yaml` equal
   `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json`. Printing twice gives equal
   bytes.
7. `an_old_reader_still_reads`. The golden file of the first release that adds the format stays in
   the tree. A test checks that every key it holds is still printed, with the same meaning, for the
   same input. An optional key added later is left out when it is empty, as `ess-ui-check/1` does
   (`website/docs/reference/formats.md:219`).

Also:

- `website/docs/reference/formats.md` has a row `UI resolved document: format: ess-ui-resolved/1`.
  It names the release that added it, says Serialize-only, says what keeps and what changes the
  version, and says that `document` uses the keys of `ess-ui/1` and holds every default.
- `crates/edge/ess-xtask/src/docs.rs` `FORMAT_RELEASES` (`:396-399`) gets
  `("ess-ui-resolved", 1, Some("<release>"))`.
- The `ess ui load` CLI reference (`website/docs/reference/cli.md:1855-`) lists `--format`.
- `task test-xtask` and `task ci-lint` pass. `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555` (branch `integration/wave-2026-11-24`, workspace 0.98.0). Nothing under
`crates/ui`, `schemas/ui` or `crates/edge/ess-cli/src/ui.rs` changed since `fad42d26f`, where the
earlier draft (never filed in the store) was read. The request came from a downstream application on
2026-11-24 (report item 3, its finding F3).

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
     | model binding | 195 | removed by `g03-resolved-binding` |
     | live semantics | 134 | kept (see below) |
     | tokens | 91 | kept (see below) |
     | fixtures | 79 | kept (see below) |
     | expressions | 35 | kept (see below) |

     The requester estimates that `ess ui load --format json` with the bindings resolved removes
     about 960 lines. The four "removed" rows above sum to 949.
   - The requester also states that the ESS crates are not published (`Cargo.toml:61`,
     `publish = false`) and that the typed document cannot be serialised. So linking the crates is
     no alternative for them. This supports P0-D8 (A) and rejects alternative (a) below.
   - Consequences for the units:
     - Short forms are expanded as well as defaults filled. `document` is the tree after step 3 and
       the fill.
     - Page resolution is in `document`.
     - The model binding is part of P0, not a follow-up: `g03-resolved-binding` ships with this
       story's release.
2. **Class.** This is a *gap*. The resolved form exists only as Rust values. A Rust generator can
   link `ess-ui`; a generator in another language has no route. `ess ui check --format json` prints
   findings, not the tree (`formats.md:219`).
3. **Already expressible?** No. The route table is serialised into every generated React project as
   `src/binding.ts` (`crates/ui/ess-ui-react/src/assets.rs:286-296`), but reaching it means running
   the React generator and parsing TypeScript.
4. **Fit.**
   - No language grid applies (`docs/design/language-grids.md:15`, `:61-63`). No authored key is
     added.
   - The document part is the loader's own expanded tree in `ess-ui/1`'s vocabulary: not a new
     shape and not a property bag (`AGENTS.md` § Boundaries).
   - `--format json` and the `ess-ui-<what>/1` naming follow `ess ui check --format json` and
     `ess ui test --format json` (`formats.md:219-221`).
   - Every target keeps reading the same in-memory values, and the round-trip test holds the
     printed form to them.
5. **Second adopter.**
   - A team generates a native mobile client from the same document.
   - A test-planning tool lists every addressed node path, so its `ess-ui-test/1` steps name only
     nodes that exist.
6. **Cost.**
   - One new Serialize-only output format, `ess-ui-resolved/1`. It is a persisted interchange
     format, because a downstream generator pins it. The duty is a golden file, an old-reader test
     and a row in the formats reference.
   - A key `ess-ui/1` adds later appears in `document` only when the document uses it or the
     schema gives it a default, so the format stays `/1`. A renamed or removed key needs `/2`.
   - A changed schema default changes resolved bytes, not the shape. That is a change of meaning
     of `ess-ui/1`, and it moves the golden file visibly.
   - One flag on `ess ui load`. No diagnostic, no migration.
   - For contrast, the compiled `EssIr` is deliberately unversioned, with no persisted reader
     (`formats.md:204`). That fits an output nobody pins. This output exists to be pinned.
7. **Considered.**
   - (a) Change nothing: a Rust generator links `ess-ui`. Rejected: other languages get no route.
     The crates are unpublished and `Document` cannot be serialised, which the requester's own
     generator confirms.
   - (b) Derive `Serialize` on every type in `model.rs` and print `Document`. Rejected: about 3,700
     lines of types would need symmetric serde attributes, and the schema would get a second
     source of key spelling.
   - (c) A new purpose-built shape. Rejected: new vocabulary for one request.
   - (d) Chosen: the expanded and filled tree in `ess-ui/1` keys, plus the node index.
   - From the request: the flag, "every addressed node with its node path", "every default filled
     in", the format id and the pin are taken. "Resolved to its … type" is changed into the route
     table (`g03-resolved-binding`, P0-D11). "`ess generate ui` reads the same form" is changed into
     a round-trip test (P0-D10).

## Decisions

**Accept, redesigned.** The defaults question of the earlier draft (its D3) is settled by the
owner as "the loader fills every default". The open questions:

- **P0-D8 (the draft's D1): Is the printed form a versioned format?**
  - (A) Yes: `ess-ui-resolved/1`, with a formats row, a golden file and an old-reader test.
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
  templates belong to the downstream application. The built-in token table is published as data in
  the schema (`constructs.Tokens.builtins`), and an application that uses its own themes does not
  need it. No later story.

## Scope

- cited: `crates/ui/ess-ui/src/lib.rs` (expose the filled tree beside `Document` from `load`,
  111-126; a `resolved` function next to `check`, 166-175)
- cited: `crates/ui/ess-ui/src/path.rs` (`Located`, `NodeRef`: a construct name per variant)
- cited: `crates/edge/ess-cli/src/ui.rs` (`Load` 31-41 gains `--format`; `run` 105-115)
- new: `crates/ui/ess-ui/tests/resolved.rs`, `crates/edge/ess-cli/tests/ui_load_json.rs`,
  `crates/edge/ess-cli/tests/golden/partner-portal.resolved.json`
- `website/docs/reference/formats.md`, `website/docs/reference/cli.md`,
  `crates/edge/ess-xtask/src/docs.rs`, `CHANGELOG.md`
- Overlap: no open story covers a resolved UI form or `ess ui load` output.
  `story:ui-binding-contract` (implemented) made `Binding` serialise.
- Size: M, about 420 changed lines (code about 150, tests about 220, docs about 50).

## Depends on

- `g03-loader-fills-defaults` (the fill step and the exposed tree)

`g02-document-digest`, `g03-resolved-binding` and `g03-model-defaults` build on it. The golden file
moves with each later default unit (B, A, P, M, R), and each of those re-pins it, naming its cause.

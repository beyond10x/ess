---
format: aep.planning-md/3
id: story:each-node-of-the-resolved-ui-document-says-whether-it-is-a-host-slot-so-a-precondition-over-a-document-is
kind: story
status: draft
title: Each node of the resolved UI document says whether it is a host slot, so a precondition over a document is evaluated from one JSON output
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
revision: 1
---
## Need

A tool that rewrites an `ess-ui/1` document (a transform that moves a section, swaps a component
or renames a field) has preconditions over the document it rewrites: a node exists at a path, the
node at a path is a section, the node at a path is not a host slot. A downstream application's
specification models such transforms. It holds their preconditions as free text on a row, and the
command that applies a transform answers "the precondition does not hold" with an `external:`
branch. The requester's words: "ESS has no predicate over an ess-ui/1 document".

Confirmed at `bcfc51555`, and where each fact can be read today:

- The specification language has no notion of a UI document. `ess-domain` and `ess-compiler`
  depend on `ess-primitives` and `ess-yaml` only (`crates/specify/ess-domain/Cargo.toml:12-15`,
  `crates/specify/ess-compiler/Cargo.toml:12-19`). `crates/ui/ess-ui` depends on nothing of ESS
  but the leaf `ess-yaml` (`crates/ui/ess-ui/Cargo.toml`; `AGENTS.md` § Crate tree).
- A document a command names is data outside the store, so in the specification the transform's
  verdict is correctly `external:`. The specification says what the evaluator decides, and the
  evaluator is the application's (`docs/design/stored-rules-boundary.md:34-51`, the same shape
  for rules held as data).
- "A node exists at P, of construct C": the Rust crate answers it (`Document::nodes`,
  `crates/ui/ess-ui/src/path.rs:108-110`), and no CLI output does. The report's item 3
  (`ess ui load --format json`, format `ess-ui-resolved/1`, P0's story from request r2) adds
  `nodes: [{path, construct}]`.
- "The node at P is not a host slot": `ess ui check --format json` already prints `hosts`, every
  host slot by canonical path (`website/docs/reference/formats.md:219`, `ess-ui-check/1`, since
  0.91.1). `host` sits on a page, a section, an overlay, a tab's form node or a field
  (`crates/ui/ess-ui/src/model.rs:605`, `:2545`; `CHANGELOG.md:401`).

So every fact is printed, but by two commands in two formats. This story puts them in one entry
per node.

## Outcome

Every entry of `ess-ui-resolved/1`'s `nodes` that is a host slot carries `host: true`. The member
is left out on every other node. The `ess-ui/1` reference gains a short section, "Evaluating a
precondition over a document": exists, is of construct C, is not a host slot, each read from
`nodes`. It also says why the specification's command keeps such a verdict as `external:`. No
predicate language, no new verb, no new format.

## Acceptance

- Red first: an `ess-ui` test (or the `ess ui load --format json` CLI test of the item 3 story)
  loads a document with a host slot section and expects its entry
  `{"path": "pages/<p>/sections/<s>", "construct": "section", "host": true}`. Entries for nodes
  that are not slots have no `host` member. Before this story the entry holds `path` and
  `construct` only.
- One case per carrier: page, section, overlay, tab form node, field.
- For every committed document under `examples/` and the `ess-ui` test corpus, the paths flagged
  `host: true` equal the paths `ess ui check --format json` lists under `hosts`.
- The item 3 story's golden file of `examples/partner-portal/ui.yaml` is unchanged when that
  document has no host slot. Otherwise the change is exactly the added members.
- `website/docs/reference/formats.md`'s `ess-ui-resolved/1` row names the member. The `ess-ui/1`
  reference has the section above.
- `crates/edge/ess-xtask/tests/layout.rs` still passes: no specification crate depends on
  `ess-ui`, and `ess-ui` depends on nothing new.

## Fit review

Tree `bcfc51555`. Received 2026-11-24 from a downstream application, report item 17.

1. **Need.** Evaluate structural preconditions of a document transform (existence, construct, host
   slot) against a document by machine, not from text. No syntax was proposed.
2. **Class.** *Gap* in the CLI output, which is small. As a specification-language construct it
   is *out of the language*: the document is data outside the store, and its evaluator is the
   application's.
3. **Already expressible?** Partly. Host slots come from `ess-ui-check/1` `hosts`, and existence
   and construct come from the item 3 story's `nodes`. This story joins the two.
4. **Fit.** The language grids do not cover `ess-ui/1` (`docs/design/language-grids.md:15`). The
   member reuses `ess-ui/1`'s own word, `host`. It composes with item 3's node index and with
   `ess-ui-check/1`, which keeps its `hosts`. The specification side is unchanged: the command
   keeps `external:`, whose verdict the application computes from the JSON.
5. **Second adopter.** A codemod that renames a section in many documents checks first that the
   path exists and is no host slot. A test-plan generator writes `ess-ui-test/1` steps only for
   nodes that exist and that ESS draws.
6. **Cost.** One optional member in `ess-ui-resolved/1`. If the item 3 story has not been
   released, the format is born with it. If it has, an additive optional member keeps `/1`, under
   that story's own rule. No source format, no diagnostic. Size S.
7. **Considered.** See `P2B-D4`. (b) An `ess ui assert` verb reading a small assertion format
   (`{at, exists, construct, host}`) and printing a verdict per assertion: a new published format
   for checks the application can make from `nodes` in a few lines. (c) A guard over a UI document
   in the specification language: `ess-domain` would depend on `ess-ui`, synthesis would have to
   generate UI documents to witness both sides, and every target (Rust, Go, TypeScript, Entity
   Runtime) would need a UI loader. That crosses the crate boundary and is a second runtime of a
   kind. (d) Decline: the application keeps text plus `external:`. (a) Chosen.

## Decisions

**Accept, redesigned** (`P2B-D4` (a)).

- **D1 (`P2B-D4`). Where is a predicate over a UI document evaluated?** (a) By the application,
  over `ess-ui-resolved/1`, with this host member, while the specification keeps `external:`.
  (b) By an `ess ui assert` verb with an `ess-ui-assert/1` assertion file, inside `ess-ui`. (c)
  By a specification-language construct. (d) Decline. **Recommended: (a).** (b) becomes the next
  step if a second tool needs the same assertions. (c) crosses the boundary that keeps `ess-ui`
  free of the specification crates.

## Scope

- inferred: the serializer of the item 3 story's `nodes` entries in `crates/ui/ess-ui/src/`
  (beside `path.rs` `Located`)
- cited: `crates/ui/ess-ui/src/model.rs` (`host` at 605, 2545, and the other carriers)
- inferred: the item 3 story's golden file and CLI test
- cited: `website/docs/reference/formats.md` (`ess-ui-resolved/1` row), the `ess-ui/1` reference
- Size: S (about 120 changed lines).

## Depends on

- P0's story for report item 3 (`ess ui load --format json`, `ess-ui-resolved/1`; request r2)

## Source

A downstream application, 2026-11-24, report item 17.

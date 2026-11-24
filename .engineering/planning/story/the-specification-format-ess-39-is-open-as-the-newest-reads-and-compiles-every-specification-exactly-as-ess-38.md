---
format: aep.planning-md/3
id: story:the-specification-format-ess-39-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess-38
kind: story
status: archived
title: The specification format ess/39 is open as the newest, reads and compiles every specification exactly as ess/38 does, and holds the gates the digest functions and declared readers use below it
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-11-24T03:24:01Z", actor: "human:timo", revision: 2}
---
## Need

`g15-digest-of-inputs` (report item 15) and `g16-entity-readers` (report item 16) each admit new
source surface: two functions in a command guard, and a key on an entity. Every source change
lands behind a format gate (`AGENTS.md` § Determinism and formats). By policy 2 neither joins
`ess/38`: that format opens the member address of a struct input, and these constructs are not on
its row or column (`docs/design/generic-language-features.md:596-601`; operator decision
`P2B-D5`). Opening a format moves pins in about seven places beside the gate. Wave 2026-11-24's
unit s0 measured this for `ess/38`
(`story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`).
So one unit opens the next format ahead of both constructs, and they only add surface behind its
gate.

## Outcome

- `ess specify formats` lists `ess/39`, unreleased and newest. `ess/38` keeps its row byte for
  byte as released.
- A specification declaring `ess/39` validates and compiles exactly as at `ess/38`: the IR is
  equal apart from `format`, and `examples/billing` gets the same refusals at both.
- `FormatVersion::V39` exists, with one gate helper per construct that answers `true` from
  `ess/39`, and one helper each that builds the `unsupported_format_version` refusal naming
  `ess/39`. Nothing calls them yet.
- The format documents and pins name `ess/39` as the newest.

## Acceptance

- Red first: `crates/specify/ess-domain/tests/format_history.rs` gains a case asserting that
  `ess/39` is supported and newest and that its history row has `release: None`. It fails before
  the change.
- `crates/edge/ess-cli/tests/epic_formats_open.rs` (or its successor) asserts that
  `examples/billing` at `ess/39` and at `ess/38` compiles to IR that differs only in `format`.
- The pins that name the newest format move once, here: `synthesis-sources.digest`,
  `synthesis-suite-bytes-table.tsv`, `synthesis-component-suites-table.tsv`,
  `lowered-definitions-table.tsv`, `models/toolchain/domains/specify.yaml`,
  `website/docs/reference/formats.md`, `website/docs/reference/spec-versions.md`,
  `website/docs/start/first-specification.md`. Each moved row names "ess/39 opened".
- No suite pair and no delta version opens. The suite major follows vocabulary
  (`ConformanceSuite::select_fresh_format_for`, `crates/verify/ess-conformance/src/scenario.rs`).

## Fit review

Tree `bcfc51555`. Not an adopter-facing construct: the format step that two constructs from
report items 15 and 16 need. Questions 1-5 do not apply. 6 (cost): one unreleased format, and the
pins listed above. 7 (considered): (a) put both constructs into `ess/38` before its release: one
format less for adopters, but against policy 2; (b) one format per construct: against policy 2
too ("a construct for one request does not get a format of its own"); (c) chosen: one shared next
format.

## Decisions

- **D1 (`P2B-D5`). Do the constructs join `ess/38` if they land before its release?** (a) Yes,
  and this story is dropped. (b) No: `ess/39`, opened by this story. **Recommended: (b)**, as
  policy 2 is written. (a) needs the operator to depart from it.

## Scope

- cited: `crates/specify/ess-domain/src/system.rs` (`SUPPORTED_FORMATS`, `FORMAT_HISTORY`,
  `FormatVersion`)
- cited: the eight pin files in Acceptance, as the `ess/38` opening unit lists them
- inferred: `crates/specify/ess-domain/tests/format_history.rs`, `crates/edge/ess-cli/tests/epic_formats_open.rs`
- Size: S (about 200 changed lines).

## Depends on

- `ess/38` opened and released (wave 2026-11-24 unit s0 and the release after it). If `ess/38` is
  still unreleased when this lands, D1 decides.

## Source

A downstream application, 2026-11-24, report items 15 and 16.

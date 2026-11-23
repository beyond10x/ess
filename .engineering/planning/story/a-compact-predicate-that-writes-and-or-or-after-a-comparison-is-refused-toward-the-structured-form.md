---
format: aep.planning-md/3
id: story:a-compact-predicate-that-writes-and-or-or-after-a-comparison-is-refused-toward-the-structured-form
kind: story
status: draft
title: A compact predicate that writes `and` or `or` after a comparison is refused toward the structured form
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-compiler/tests/expression_validation.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/stored_field_guards.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/quoted_predicates.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/direct_response.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: release-notes/
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 12
---
## Outcome

A compact predicate holds one comparison. When the unquoted right side of a comparison contains
the word `and` or `or`, or a second comparison operator, ESS refuses it as `unparsable_predicate`
(`ESS-SPEC-012`). Today it reads the rest of the string as one text literal. The message names
structured `all`/`any`/`not` and the quoted spelling, as the refusal of `&&` and `||` already does.
The canonical rendering quotes a text literal that holds such a word, so the IR's own text reads
back as the same predicate.

## Need

Our own finding, made while assessing other reports. Nobody requested it, so there is no fit
review. Reproduced on the released 0.97.3 binary with the generic model `f1/template.yaml`.
`f1/probe.sh` puts one spelling into one place and leaves every other place at a valid baseline.
`f1/matrix.sh` runs every case. All output is in `f1/run.txt`.

- `when: sku == A1 and gift` validates (`shop v1 — 1 file(s), valid`) and compiles to
  `{"kind":"when","predicate":"sku == A1 and gift"}`. That is byte for byte what the quoted
  `sku == "A1 and gift"` compiles to. The structured `{all: [sku == A1, gift]}` compiles to
  `{"kind":"when","predicate":{"all":["sku == A1","gift"]}}`.
- Synthesis reads the spelling as a literal. For `placed` the suite sends
  `{"sku":"A1 and gift","gift":true}`. The structured form sends `{"sku":"A1"}`
  (`f1/cases/w-and/suite.json` beside `f1/cases/w-structured/suite.json`).
- A compact `where:` with `input.`: `instances: {where: team == input.team and gift == true}`
  compiles to `"filter":"team == \"input.team and gift == true\""`. That is a bulk move of every
  row whose `team` is that whole text, so in practice no row.

The family. The left side is a `String`, or a newtype or `Optional` of one. Every place admits it:

| spelling | where | 0.97.3 reads it as |
|---|---|---|
| `sku == A1 and gift` | `when:` | `sku == "A1 and gift"` |
| `sku == input.team and gift == true` | `when:` | `sku == "input.team and gift == true"` |
| `sku == A1 or sku == B2` | `when:` | one comparison with `A1 or sku == B2` |
| `sku == A1 and not gift` | `when:` | one comparison with `A1 and not gift` |
| `sku != A1 and gift` | `when:` | `!=` with `A1 and gift` |
| `sku == A1 or channel in [Web]` | `when:` | one comparison with `A1 or channel in [Web]` |
| `sku == (A1 and gift)` | `when:` | one comparison with `(A1 and gift)` |
| `sku == A1 == B2`, `sku < m < z` | `when:` | one comparison with `A1 == B2`, `m < z` |
| `sku == A1 and team > B` | `when:` | one comparison with `A1 and team > B` |
| `sku == A1 AND gift` | `when:` | one comparison with `A1 AND gift` |
| `sku == A1 and gift` inside `all: [...]` | `when:` | one leaf, a literal |
| `t == A1 and gift` | a quantifier's `that:` | one comparison |
| `sku == input.sku and gift == true` | `when_related: {entity, where}` | literal |
| `team == input.team and gift == true` | `instances: {where:}` | literal |
| `team == subject.team and gift == true` | an `affects:` entry's `where:` | literal |
| `sku == input.sku and state == Open` | `when_subject: {predicate:}` | literal |
| `sku == A1 and state == Open` | a view's `filter:` | literal |
| `team == A or team == B` | an entity invariant | literal |
| `label != X and code != Y`, `value != X and value != Y` | struct and newtype invariants | literal |
| `event.message_id == A1 and event.kind == ship` | a binding cause's `where:` | `event.message_id == "A1 and event.kind == ship"` |

Already refused, but for the wrong reason or with the wrong hint:

| spelling | 0.97.3 says |
|---|---|
| `channel == Web and gift` (enum left side) | `ESS-COMMAND-001` "compares `channel` to `Web and gift`, which … does not declare as an enum variant" |
| `gift == true and sku == A1` (Boolean) | `ESS-COMMAND-002` "`Text literal `true and sku == A1``" |
| `n < 0 or n > 100`, `n == 1 or n == 2` (Integer; workbench repros 18a and 18c under `.engineering/repro/workbench-2026-11-09/examples/`) | `ESS-COMMAND-002` and `ESS-VIEW-002`, "`Text literal `0 or n > 100``" |
| `(sku == A1) and gift`, `not (n == 1 or n == 2)` (repro 18b) | `ESS-SPEC-012` "left-hand side must be a fact path: invalid fact path identifier \"(sku\"". No hint toward `all`/`any` |

Correctly refused today: `sku == A1 && gift` (`ESS-SPEC-012` "`&&` and `||` are not part of the
compact form … use structured all/any/not"), and `sku != "" and team != ""` ("tokens after a
quoted operand are unsupported; use structured any/all/not").

Why a modeller writes it. ESS prints conjunctions in exactly this spelling. Diagnostics and
generated documents render `all` as `(a and b)`, for example "every input satisfying
`(route == X and flag_b == true)`" and "Taken when the existing subject's stored fields satisfy
`(service == Express and weight_kg > 20)`" (`write_joined(…, " and ")` in
`crates/specify/ess-primitives/src/predicate.rs`). Copied without the parentheses, the text
validates and means something else. The repository shows the same misreading in three places:

- the test input `"service == Express and service == Standard"` in
  `crates/specify/ess-domain/tests/stored_field_guards.rs:397`, refused today only because
  `Express and service == Standard` is no enum variant;
- `"value.count < 9 and value != ''"` in `crates/verify/ess-conformance/src/direct_response.rs:473`,
  a unit test whose name says it is "an invariant over the value alone";
- the planning text of `story:caller-attributes-are-readable-in-a-when-related-predicate` (item 5:
  `where: device_id == caller.device_id and state == Open`).

The reference already states the rule: "A compact predicate is one string. It holds a single
comparison, a bare path, a presence check or a `not` in front of one of those"
(`website/docs/reference/predicates.md:167`). It also says "The compact form has no `&&`, `||` or
`in [...]`" (`:245`). Only `&&` and `||` are enforced (`COMBINATORS`, `predicate.rs:1381`). The
words `and` and `or` are not.

## Design

The parser reads a compact comparison's unquoted right side and refuses it when one of its
whitespace-separated tokens is a word combinator (`and`, `or`, `AND`, `OR`, decision D2) or a
comparison operator (`==`, `!=`, `<`, `<=`, `>`, `>=`). The refusal is the existing
`ParseError::predicate`, reported as `unparsable_predicate` (`ESS-SPEC-012`) at the declaration,
like the `&&` refusal (`predicate.rs:3858-3873`). Its message is: "`and` and `or` are not part of
the compact form, so `<operand>` is not one operand; use structured all/any/not to combine
predicates, or quote \"<operand>\" to compare with that text". A chain gets "a compact predicate
holds one comparison".

- Every place shares one parser (`Predicate::parse_expression_nested`), so every row of the family
  table changes together. That includes a binding's `where`, which goes through the same leaf
  parser.
- An Integer, Boolean or enum left side is refused by the parser before the type check, so it gets
  the structured hint instead of a type mismatch with a text literal. A left side that starts with
  `(` gets the same hint added to its existing "must be a fact path" message.
- A quoted literal stays text: `sku == "A1 and gift"` is admitted, as `sku == "A1 && gift"` is.
- `Operand`'s `Display` quotes a text literal holding one of these tokens, as it quotes `&&` today
  (`predicate.rs:1383-1401`). The IR then renders `sku == "A1 and gift"`, which reads back.
- Not refused: `not` and `in` as words in a text (`note == do not disturb`, `label == in stock`).
  A combination is always reached through `and` or `or`, so they are not needed for the defect.
- The Go and TypeScript runners read IR leaves (`src/go/predicate.go` `parseLeaf`,
  `src/ts/predicate.ts`). The compiler stops writing the unquoted spelling, and both already read
  a quoted operand. No runner change is needed (inferred; checked by acceptance 5).

Breaking-change analysis:

- Committed models newly refused: none. All 365 model roots under `examples/` and
  `crates/**/tests/fixtures/` at tag `0.97.3` were compiled (353 compile, `census/census.sh`). Their
  IR was scanned for a comparison leaf whose right side holds `and`, `or`, `not`, `in` or a second
  operator, quoted or not. 0 hits (`census/swallow.txt`, `census/swallow-quoted.txt`). A `git
  grep` of every YAML file finds only the three workbench repros, and they are already refused.
- Rust test inputs that change: 2. `stored_field_guards.rs:397` is rewritten to the `all:` form it
  means. The input at `direct_response.rs:473` is rewritten to `{all: [...]}`. Both meant a
  conjunction.
- Adopter models: a model that validated with such a spelling is refused. Every such model
  compared a text field with a sentence, which its author almost certainly did not mean. An
  artifact (`ir.json`, a suite) written by 0.97.x that carries such a literal unquoted must be
  regenerated. A model whose quoted literal holds one of these words changes model digest, because
  the rendering gains quotes. That is the same consequence #93 had for `&&` and `||` ("Artifacts
  written by 0.32.x that carry such literals must be regenerated").
- Format: AGENTS.md § Determinism and formats asks for a new `ess/N` when meaning, identity,
  references, canonicalization, names or the envelope change. No admitted document changes
  meaning. The refusal removes documents that were misread. The canonical rendering changes only
  for a quoted text literal holding a combinator word, and it parses to the same predicate. The
  precedents #93 (`&&`/`||`, `null`) and #438 (a membership operand read as literal text) refused in
  every format with no new version and a Breaking changelog entry. Recommended: the same, D3.
  `CHANGELOG.md` gets an entry starting `**Breaking**`. No suite or IR format number changes.

## Acceptance

1. Each spelling of the family table, in each place, is refused with `ESS-SPEC-012` at that
   declaration's line. The message contains "structured", "all", "any" and the quoted operand:
   `cargo test -p ess-compiler --test expression_validation`, extending
   `a_compact_conjunction_or_disjunction_is_refused_toward_the_structured_form` with `and`, `or`,
   `AND`, `OR` and a chain, in a `when:`, a row-set `where:`, `instances: {where:}`, an `affects:`
   `where:`, `when_subject.predicate`, a view `filter:`, an entity, struct and newtype invariant, a
   quantifier `that:` and a binding cause `where`.
2. `sku == "A1 and gift"`, `sku == 'A1 or B2'`, `note == do not disturb` and `label == in stock`
   validate, and the first compiles to IR text `sku == "A1 and gift"`, which parses back to the same
   predicate: the same test, plus a case in `crates/specify/ess-primitives/tests/quoted_predicates.rs`.
3. `n < 0 or n > 100` on an Integer and `channel == Web and gift` on an enum are refused with
   `ESS-SPEC-012` and the structured hint, not `ESS-COMMAND-002`/`-001`. `(sku == A1) and gift`'s
   message names `all`/`any`/`not`: same test file.
4. `website/docs/reference/predicates.md` § Compact forms says `and`, `or` and a second operator are
   refused, with `ess-check` examples expecting `refused` and `ess-says="are not part of the
   compact form"` beside the quoted form expecting `synthesizes`: `task test-xtask` (the page's
   example test).
5. The committed example trees and suites do not move: `ess-synth`'s `feasibility`,
   `adversary_related_guard_behaviour_pass1` and `http_binding_rust`, and the byte pins in
   AGENTS.md § Gate stay green unchanged.
6. `CHANGELOG.md` and `release-notes/<version>.md` carry a `**Breaking**` bullet: `cargo xtask
   release notes --check`.

## Decisions

- **D1. Refuse or parse.** (A) Refuse, as `&&`/`||` are refused. One parser change, no grammar.
  (B) Parse `and`/`or` in the compact form into `all`/`any`, with precedence and parentheses. That
  is a new grammar in the Rust parser and in the Go and TypeScript leaf readers, and a new `ess/38`,
  because the same text changes meaning. It would also make ESS's own `(a and b)` rendering
  readable back. (C) Warn and keep the literal reading. **Recommended: A.** The language keeps one
  way to combine, and B can still be added later over a refusal; it cannot be added over a silent
  literal.
- **D2. Which tokens.** (A) `and`, `or`, `AND`, `OR` and a comparison-operator token. (B) Only
  lowercase `and`/`or`. (C) Also `not` and `in`, which would refuse texts like `do not disturb`.
  **Recommended: A.**
- **D3. Which formats.** (A) Every format, no new `ess/N`, Breaking changelog entry: the #93 and #438
  precedent. (B) Refuse from the next open `ess/N` only; below it keep the literal reading with a
  warning on standard error. **Recommended: A.** No committed model is affected, and what is read
  today is wrong in every format.

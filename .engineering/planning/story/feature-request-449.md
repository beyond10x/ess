---
format: aep.planning-md/3
id: story:feature-request-449
kind: story
status: implemented
title: A regular-expression predicate
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#449
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-451
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/predicate_reference_page.rs
- confidence: cited
  path: docs/design/string-predicate-operators.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:33Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:33Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:46Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#449: A regular-expression predicate.

## Origin
beyond10x/ess#449, filed 2026-10-05; reported from an adopter's expression catalogue, which has a REGEXP operator, five regex namespaces and a reserved `name_match: pattern`. This is the third request: #95 and #103 each declined regular expressions.

## Fit review
1. Need: a guard, filter or invariant should hold only when a text value has a given shape, such as "starts with `GB-`, nine characters, letters, digits and hyphen only". The requester proposes either a regular-expression predicate in a named portable dialect (an RE2 subset) that every runner evaluates identically, with synthesis witnesses, or a design note that puts patterns out of scope. Both options are theirs. In the adopter's catalogue, REGEXP is an operator inside conditions held as data, which is #451's question, not a guard the specification itself evaluates. Minimal reproduction: `<fit-review scratch>/probe-449/` (installed ess 0.52.0). `code: {matches: "^GB-[0-9]{6}$"}` is refused with `unknown operator "matches"; expected one of eq, ne, …, starts_with, ends_with, contains, equals_ignore_case, in_ignore_case`.
2. Class: two parts.
   - Convenience, for the shapes ESS can already state: a fixed prefix, suffix or substring, a character set, a length, a closed set.
   - Gap, for position-dependent character classes ("six digits after the prefix"), which no construct states.
   - Local policy, for the catalogue's regex namespaces: they are one adopter's data vocabulary.
   - Not a defect: ESS declined regular expressions deliberately and documented why (`docs/design/string-predicate-operators.md:528-529`, `docs/design/string-alphabet-and-length.md:550-551`).
3. Existing idiom: `alphabet:` on a String newtype (ess/11), `.count` (Unicode scalar values), `starts_with`/`ends_with`/`contains` (`website/docs/reference/predicates.md:830-918`), `prefix:` on a newtype (`website/docs/guides/specify/fields-and-invariants.md:144-162`), `equals_ignore_case` (`predicates.md:919-982`) and `any_of` for closed sets. The probe guards `any: [code.count != 9, not: {code: {starts_with: "GB-"}}]` over an alphabet newtype. It validates (`probe v1 — 3 file(s), valid`), and synthesis writes 2 scenarios with 0 refusals, witnesses `GB-GB-GB-` / `ZA01ZA01Z`.
4. Fit of a regex predicate:
   - Witnesses: synthesis needs a text that matches and a near miss that does not, for every pattern. That is a generator over the pattern's automaton, plus a refutation case per construct, and today each string operator has a hand-built candidate rule (`predicates.md:1269`).
   - Dialect: Rust `regex` and Go `regexp` are RE2-family; TypeScript `RegExp` is ECMA-262 and differs on `\d`, `\w`, case folding and code units vs scalar values. The JSON Schema projection emits `pattern` only where "the grammar is small enough to be certainly right" (`crates/generate/ess-gen/src/types.rs:46`). So an RE2 subset needs ESS's own parser and a translation per lane.
   - Entity Runtime: lowering refuses the other unsupported text constructs by name (`CaseFoldUnsupported`, `predicates.md:971-972`). Whether entity-core has a pattern condition: I don't know.
   - No crate depends on `regex` directly; it is only in `Cargo.lock` transitively (`Cargo.lock:1696`).
5. Second adopter: a postal-code or account-number format check, such as "two letters, then digits", is a common domain fact. But it is a value-shape constraint, better placed on the type (an `alphabet:` and `prefix:` newtype) than in a guard. No second adopter here needs a regex guard that the existing idiom cannot approximate. Positional classes remain the only true gap.
6. Cost as proposed: source format ess/23, a pattern grammar and parser, three runtime evaluators held to shared vectors, witness generation, a new suite pair (ess-conformance/44, /45), diff classifications, a JSON Schema `pattern` projection and a named Entity Runtime refusal. Size L. Cost of the decline: one reference-page section held by an existing drift test.
7. Alternatives: (a) change nothing: the third request gets the same undocumented answer. (b) An RE2-subset `matches:` operator: refused for the witness and dialect cost against one adopter whose regexes are stored data. (c) A positional character-class construct on newtypes, such as a per-position alphabet: a smaller real gap, but no adopter has asked for it. Not built; it is named as the reopening condition. (d) Chosen: decline with the idiom, documented on the predicate reference page.

## Decisions
Decline, with the idiom. The story body is the decline record. ESS has no pattern predicate. Shapes are stated with `alphabet:`, `prefix:`, `.count`, `starts_with`/`ends_with`/`contains` and `any_of`. A regular expression that the system stores and evaluates as data is the system's own evaluator, and #451 sets that boundary. Depends on #451 (edge recorded), which creates `docs/design/stored-rules-boundary.md` with its pattern section. Reopen only when an adopter's guard (not stored data) needs a position-dependent class that the idiom cannot state; the candidate then is (c), not a regex. Built: the section `## Text shapes without patterns` on `website/docs/reference/predicates.md`, after the case-insensitive operators (:919-982), with checked blocks. Reply on the issue. No format bump.

## Acceptance
- predicate_page_text_shape_idiom_synthesizes: an `ess-check="when" ess-expect="synthesizes"` block combining `starts_with` and `.count` on the page model passes `crates/edge/ess-cli/tests/predicate_reference_page.rs`.
- predicate_page_matches_operator_refused: an `ess-check="when" ess-expect="refused"` block with `{matches: …}` passes the same test, and the refusal lists the admitted operators.
- predicate_page_text_shapes_section_names_the_boundary: `website/docs/reference/predicates.md` has the heading `## Text shapes without patterns`. The section contains "no pattern predicate", "`alphabet:`", "`prefix:`", "`.count`", "`starts_with`", "position-dependent" and a link whose target ends in `docs/design/stored-rules-boundary.md`. That note exists and has the heading `## Patterns held as data`. The case `text_shapes_section_names_the_pattern_boundary`, added to `crates/edge/ess-cli/tests/predicate_reference_page.rs`, reads the page and the note and fails on a missing heading, phrase, link or note. `task site-build` is not the check, because it does not resolve the note link.

## Scope
- website/docs/reference/predicates.md  cited — new section after the string and case-insensitive operators (830-982)
- crates/edge/ess-cli/tests/predicate_reference_page.rs  cited — runs every `ess-check` block on the page; gains the section case
- docs/design/string-predicate-operators.md  cited — out-of-scope entry (528) points at the decision

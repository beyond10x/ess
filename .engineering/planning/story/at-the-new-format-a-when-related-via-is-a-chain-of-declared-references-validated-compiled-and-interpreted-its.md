---
format: aep.planning-md/3
id: story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its
kind: story
status: active
title: 'At ess/38 a when_related via is a chain of declared references, validated, compiled and interpreted, its exists: false answering a missing row at any step'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:navigation-reaches-any-depth-in-guards
- depends_on: story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:57Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:58Z", actor: "human:timo", revision: 6}
---
## Need

A command cannot refuse on a row two declared references away from the row its input names: adding
a stop to a route is refused while the route's plan is `Frozen` (stop's route -> route's plan). A
`when_related:` `via:` is one string (`crates/specify/ess-domain/src/command/related_guard.rs:107-112`,
`RawRelatedGuard.via: Option<String>`), so a list is a parse error ("invalid type: sequence,
expected a string", measured on 0.92.0 in the parent story), and the first cut of navigation kept "a
second hop in a view or a guard" refused (`docs/design/generic-language-features.md:241-249`). The
design already decided the depth: "Any chain of declared references, as written" (answer 8,
`generic-language-features.md:727-729`).

This is the accepted, still open `story:navigation-reaches-any-depth-in-guards` (active since
2026-11-10, beyond10x/ess#564, downstream finding 10). Its acceptance targets `ess/37`, which was
released in 0.93.0 without it, and it depends on
`story:navigation-reaches-any-depth-in-view-filters-and-guards` (active, not started). This unit and
`g13-guard-chain-synthesis` are its implementation units, re-targeted to the open format; the
parent's acceptance is theirs, split in two. The parent is an umbrella from 2026-11-24: it is not
built itself, so no unit waits on it, and nothing reaches the view-navigation story through it.

Minimal reproduction (the parent's, renamed generically; not run here). `demo.ship.AddStop(route_id)`
creates a `Stop`; `Route.plan_id` references `Plan`, whose lifecycle is `Draft -> Frozen`:

```yaml
outcomes:
  - name: no-route
    when_related: {via: input.route_id, exists: false}
    error: demo.ship.RouteNotFound
  - name: no-plan
    when_related: {via: [input.route_id, plan_id], exists: false}
    error: demo.ship.PlanNotFound
  - name: plan-frozen
    when_related: {via: [input.route_id, plan_id], predicate: state == Frozen}
    error: demo.ship.PlanFrozen
  - name: added
    creates: demo.ship.Stop
    instance: stop_id
```

The requester's words, their identifiers replaced: "`AddStop` should be refused when the route's
plan is `Frozen` ... `when_related` reads one hop" (report item 13); their workaround enforces the
frozen state only at the commands one hop from it.

## Outcome

From the format named under Decisions, `when_related: {via: [<first>, <ref>, …], exists | predicate}`
validates: `<first>` is `input.<field>` or a stored field of the addressed subject, as a one-hop
`via` is; each later step is a field of the row the step before names, typed as a declared
entity's identity (or `Optional<…>` of it). `exists: false` over a chain is taken when the row is
missing at any step; a predicate reads the last row, and a missing row at any step makes it unknown,
so only an `exists: false` branch answers it. The chain answers at step 4 of the precedence rule
with every related-row guard. The interpreter and history checker read it. Below the format a list
`via:` is a named `unsupported_format_version`, not a serde error.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first: `crates/specify/ess-domain/tests/related_guard.rs` (existing binary) gains
  `a_when_related_via_chain_validates_at_the_new_format`: the reproduction validates at the new
  format; on the base it fails to parse.
- Below the format the same text is refused as `unsupported_format_version` naming the format,
  its hint `FormatVersion::declaration_hint`; one test.
- Refused at every admitted format, a test each: a step that is no field of the row before it, or a
  field that is not a declared entity's identity (`undeclared_reference` / `type_mismatch`,
  ESS-COMMAND-003 or -002, naming the step); a chain in an `all:` element and a chain whose first
  step is a list of references (`unsupported_construct`, named; out of this unit); a chain of one
  element (`via: [input.route_id]`, refused with the hint to write the string).
- A chain beside its own prefix (`no-route` over `input.route_id`, `no-plan` over the chain) is
  admitted; the missing-row answers are ordered in declaration order (step 4), pinned by a unit test
  in `crates/specify/ess-domain/src/command/precedence.rs` for the reproduction.
- IR: `ResolvedCondition::Related` gains the followed references as a field omitted when empty
  (the value source's `through`, `crates/specify/ess-domain/src/command.rs:1301-1311`, is the
  pattern). An old-reader compatibility test in an existing `ess-compiler` binary shows a model
  without a chain compiles to the same bytes; `cargo xtask schema` regenerates
  `schemas/generated/ess.schema.json` (a `RawSpecFile` change) and `task projection-check` passes.
- The interpreter answers the reproduction against a hand-built store, no synthesis (a case in an
  existing `ess-conformance` binary, `interpreted_related_guards.rs`): route missing (`no-route`),
  route present with a dangling `plan_id` (`no-plan`), plan `Frozen` (`plan-frozen`), plan `Draft`
  (`added`). The history checker reads the same.
- `website/docs/reference/predicates.md` § "A guard over another entity's row" documents the chain;
  `docs/design/language-grids.md` grid R row `when_related:` names it; the format row of
  `website/docs/reference/formats.md` and the format history in `system.rs` name it;
  `CHANGELOG.md` `### Added` gets one line with its grid row (policy 3). The parent's `ess/37` lines
  are superseded by `ess/38` (P2A-D6 A, D77, 2026-11-24).
- Every new test under 5 s or named with its time; no new test binary.
- Gate additions: `task test-xtask` (the step edits `CHANGELOG.md`) and the bounded slow lane `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`.

## Fit review

The parent's fit review applies unchanged (`story:navigation-reaches-any-depth-in-guards`
§ Fit review: gap, decided by the design's answer 8; the same `via:` list as the value source;
grid R row `when_related:` predicate × related-row column; composes with the precedence rule at
step 4; Rust and Go owed by name; Entity Runtime refusal kept by decision). Additions read on
2026-11-24 at tree `bcfc51555`:

- Report item 13 is the parent's need in another domain: covered.
- The value source's chain is limited to two references (`command.rs:2126-2129`); the guard's chain
  is any depth, as the design's answer 8 says. Lifting the value source's limit is not part of this
  unit (the view story owns view chains).
- The parent's dependency on the view story is not needed by the guard: the guard reads one row at
  a time through the store, as the value source's chain does. Decision `P2A-D7`.

## Decisions

**Accept as proposed** (the parent's decision). `P2A-D6` is answered A: `ess/38` (D77). `P2A-D7` is applied on
its default A: the parent's dependency on the view story is dropped, and the parent is delivered as
these two units.

## Scope

- cited: `crates/specify/ess-domain/src/command/related_guard.rs` (`RawRelatedGuard.via` 107-112;
  `via` resolution; `site`)
- cited: `crates/specify/ess-domain/src/command/related_value.rs` (`RelatedVia`, 19-40)
- cited: `crates/specify/ess-domain/src/command.rs` (the value source's chain, 1301-1311,
  2118-2130, reused)
- inferred: `crates/specify/ess-domain/src/command/precedence.rs`
- inferred: `crates/specify/ess-compiler/src/ir.rs` (`ResolvedCondition::Related`, from 643),
  `crates/specify/ess-compiler/src/resolve.rs`
- inferred: `crates/verify/ess-conformance/src/interpret/execute/related.rs`,
  `interpret/execute/history.rs`
- inferred: `crates/specify/ess-domain/src/system.rs`, `schemas/generated/ess.schema.json`
  (generated), `website/docs/reference/spec-versions.md` (generated)
- cited: `crates/specify/ess-domain/tests/related_guard.rs`,
  `crates/verify/ess-conformance/tests/interpreted_related_guards.rs` (existing binaries)
- Size: L (about 600 changed lines). If it runs past 650, the interpreter and history checker move
  to `g13-guard-chain-synthesis`.
- Overlap: s1 and s2 of the wave of 2026-11-24 change `related_guard_ir.rs`, `resolve.rs`, `ir.rs`
  and `spec.rs` (a member address in `via`); this unit lands after s2 and composes with it (a
  chain whose first step is a struct member is admitted only if s1's `address` answers it; else
  refused by name).

## Depends on

- Derived from `story:navigation-reaches-any-depth-in-guards` (parent, active, an umbrella); no
  dependency on it.
- `story:at-ess-38-instance-and-each-in-name-a-member-of-a-struct-input-and-the-documents-list-every-addressing`
  (s2, shared files).

## Source

A downstream application, 2026-11-24, report item 13 (its G8); github:beyond10x/ess#564
(downstream finding 10), the parent's source.

## Downstream report 2026-11-26

A downstream application reported on 0.99.2 (gap 4 of a report of ten) that a guard cannot reach a
second row through a first (a child, then its parent), so its callers send both identities and each
row is read through an input of its own. On `main` at `c0b5f560cb` a `when_related:` `via:` reads
one row an input names, or one row a stored field of the addressed subject names
(`website/docs/reference/predicates.md` § "A stored reference", `ess/22`), and a stored-field `via`
stays the command's only related row (§ "Several related rows"). The chain this story opens covers
the report, `via: [input.<child>, <parent reference>]`. The report adds one case to the
acceptance: a command whose input names only the child, with a guard reading the held state of
that child's parent.

---
format: aep.planning-md/3
id: story:caller-attributes-walk-every-caller-read-of-a-command
kind: story
status: draft
title: caller_attributes walks every caller read of a command
summary: ess-gen http::caller_attributes misses caller reads in an external branch's when_subject and in an error payload (beyond10x/ess#740)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#740
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

`ess_gen::http::caller_attributes` (`crates/generate/ess-gen/src/http.rs:170-247`) says it returns
"every `{caller: <attribute>}` of its values and every `caller.<attribute>` of its guards". The
`OpenAPI` projection publishes the result as each operation's `x-ess-caller`
(`crates/generate/ess-gen/src/openapi.rs:849`). Its guard walk is a hand-written match ending in
`_ => {}`, and its value walk reads only `payload` and `sets`. It therefore misses four positions
where a command reads its caller, all of which 0.92.0 validates and the interpreter reads:

1. the predicate and the input guard of an `external:` branch narrowed by `when_subject:`
   (`ResolvedCondition::ExternalSubject`);
2. `{caller: …}` in an error's payload (`ResolvedOutcome::error_payload`);
3. the input guard (`when:`) beside `when_related: {via, …}` (`ResolvedCondition::Related`);
4. the input guard beside a row-set `when_related: {entity, where, …}`
   (`ResolvedCondition::RelatedSet`).

Minimal reproduction, ess 0.92.0, `ess generate --path spec --kind openapi`. Actor
`example.desk.Author` has attribute `owner_id`. `Open` reads it in `sets:`, `Stamp` in an event
payload, `Veto` in `when_subject: {predicate: owner_id == caller.owner_id}` on an `external:`
branch, and `Lock` in `error payload: {by: {caller: owner_id}}`. `Claim` and `Audit` read it in
`when: caller.owner_id == input.owner_id` beside a `when_related:` via and a row-set
`when_related:`.

```text
/desk/commands/Audit: x-ess-caller: (none)
/desk/commands/Claim: x-ess-caller: (none)
/desk/commands/Lock: x-ess-caller: (none)
/desk/commands/Open: x-ess-caller: owner_id
/desk/commands/Stamp: x-ess-caller: owner_id
/desk/commands/Veto: x-ess-caller: (none)
```

`Veto` still answers `'403'`, "the caller is not one this branch admits: its guard compares the
authenticated caller with the input or the record". That status comes from `decided_by_caller`, not
from this walk. So the operation states that a caller guard decides it, yet lists no caller
attribute.

## Acceptance

- `ess generate --kind openapi` on the reproduction lists `x-ess-caller: [owner_id]` on `Veto`,
  `Lock`, `Claim` and `Audit`. `Open` and `Stamp` are unchanged.
- A test in `crates/generate/ess-gen/tests/caller_values.rs` asserts
  `caller_attributes(ir, command) == ["owner_id"]` for each of the four commands.
- `caller_attributes` takes its guard positions from
  `ess_compiler::expression::predicate_sites` (or a per-command form of it), not from its own
  match. A guard position added later to that walk then needs no change here. Its value walk
  destructures `ResolvedOutcome` without `..` over the value-bearing fields (`payload`,
  `error_payload`, `sets`, `affects`, `instances`), so a new value-bearing field fails to compile
  until it is walked.
- `cargo test -p ess-gen` passes, and so does every committed generated `OpenAPI` document
  (`ess generate --check`) whose model does not read its caller in those four positions.

## Scope

- cited: crates/generate/ess-gen/src/http.rs (`caller_attributes`, lines 170-247)
- cited: crates/specify/ess-compiler/src/expression.rs (`predicate_sites` line 261, `command_sites` line 354: expose a per-command walk or filter the whole-IR one by `ConstructRef`)
- cited: crates/generate/ess-gen/tests/caller_values.rs (new cases beside `the_openapi_operation_names_the_caller_attributes_it_reads`)
- cited: docs/design/language-grids.md (grid R: fill the caller cells this triage measured. `when_subject:` and error `payload:` read the caller (yes). A `when_related:` predicate and a row-set `where:` refuse it (no, `ESS-COMMAND-003`))
- inferred: CHANGELOG.md (Fixed entry)
confidence: high

## Fit review

1. **Need.** The `OpenAPI` contract must name every caller attribute an operation reads, so a
   client knows what its credential has to carry (`docs/design/caller-values.md:86`). Four
   positions are not walked, and the published contract says the operation reads nothing of the
   caller (`repro/run.txt` §1 and §2). The requester asked for the two positions they found, and
   noted two more found by reading. Both readings are confirmed by running.
2. **Class.** Defect. The function's doc comment (`http.rs:170-171`) and the design row
   (`caller-values.md:86`, "every operation names the caller attributes it reads") promise all
   reads.
3. **Already expressible?** No. The specification is correct, and the projection is the only
   source of `x-ess-caller`. No authored workaround exists short of adding an unneeded caller read
   in `sets:`.
4. **Design fit.** Grid R, caller column (`docs/design/language-grids.md`): the target is "every
   command position", and this change reads exactly the positions validation admits today.
   `ess specify validate` admits a caller in `when_subject:` and in an error payload, and refuses
   it in a `when_related:` predicate and a row-set `where:` with `ESS-COMMAND-003`
   (`repro/run.txt` §3). Reusing `predicate_sites`, the walk that "a consumer that must refuse a
   construct wherever a predicate reads it asks … so a position cannot be forgotten in one
   consumer and remembered in another" (`ess-compiler/src/expression.rs:253-260`), applies the same
   rule Entity Runtime already uses for caller reads (`ess-entity-runtime/src/lib.rs:4365`,
   `reads_caller` over a `PredicateSite`). When the caller column opens in a related predicate,
   this projection follows without a change.
5. **Second adopter.** `example.ticket.Assign` with `when_subject: {predicate: team_id ==
   caller.team_id}` on an `external:` branch "the scheduler declines", or a refusal whose error
   carries `{requested_by: {caller: agent_id}}`. Both publish no `x-ess-caller` today.
6. **Cost.** No source format, no IR change and no new diagnostic. Generated `OpenAPI` documents
   for models that read their caller only in those positions gain an `x-ess-caller` list, which is
   an annotation (`caller-values.md:86`) and not a versioned format. `ess-compiler` gains at most a
   public per-command form of `command_sites`.
7. **Alternatives.** (a) Change nothing: the contract under-reports, against its own doc. (b) Add
   the four missing match arms and `error_payload` to the existing walk, as the issue implies: this
   fixes today's case, and the next condition variant is forgotten again behind `_ => {}`.
   (c) Chosen: read guards from the compiler's one predicate-site walk, and values through an
   exhaustive destructure.

## Decisions

Accept, redesigned. The behaviour the requester expects (`["owner_id"]` for both commands) is
adopted, along with the two positions found by reading. The change from the request is that the
fix is not more match arms: `caller_attributes` takes its guard positions from
`ess_compiler::expression::predicate_sites`, so this projection and Entity Runtime cannot disagree
on where a caller is read. No format change. No design page is needed first: `caller-values.md:86`
already decides it. The same change fills the grid R caller cells in `language-grids.md`.

## Source

github:beyond10x/ess#740

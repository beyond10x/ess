---
format: aep.planning-md/3
id: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence
kind: story
status: implemented
title: The interpreted target publishes an absent event field in its declared presence
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#765
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_command_execution.rs
- confidence: cited
  path: docs/design/wire-presence-json-prefix.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:41Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:42Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-11T00:47:20Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Need

The interpreted target leaves an absent `Optional` event field out of the published event whatever
the field declares. Under `presence: null_when_absent` the specification says the field is sent as
`null`, the suite's payload leaf refuses a left-out value, and the interpreted target fails the
suite synthesized from the same specification.

Minimal reproduction (`format: ess/25`): an `Optional` input copied into a `null_when_absent`
event field, which the witness sends absent.

```yaml
events:
  - name: example.token.TokenContributed
    fields:
      - {name: token_id, type: example.token.TokenId}
      - {name: note, type: Optional<String>, presence: null_when_absent}
commands:
  - name: example.token.ContributeToken
    input:
      - {name: token_id, type: example.token.TokenId}
      - {name: owner, type: String}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {owner: input.owner}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed: {token_id: input.token_id, note: input.note}
```

`ess` 0.92.0:

```
$ ess verify conform run --path spec-input-null_when_absent --target interpreted --report-format 2
  failed example.token.ContributeToken/outcome/contributed
  8 scenarios: 7 passed, 1 failed, 0 error, 0 unsupported
      expected:
      - example.token.TokenContributed.note holds a String
      observed:
      - note was not carried
```

The same happens for a field read from the row (`agent: {subject: agent}` into
`TokenWithdrawn.agent` declared `null_when_absent`, the row holding `agent` absent): observed
`agent was not carried`.

The interpreter already follows the declared presence elsewhere: a view row's absent member is
written as `null` except under a carried `omitted_when_absent`, and a response field is checked
against its policy.

## Acceptance

- `ess verify conform run --path <the reproduction> --target interpreted --report-format 2` exits 0
  with `8 scenarios: 8 passed`.
- An interpreter test: executing `ContributeToken` without `note` publishes `TokenContributed`
  with `note` present and `null`; with the field declaring `omitted_when_absent` or no presence the
  event has no `note` member.
- With `story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null` merged, the
  row-read reproduction (`TokenWithdrawn.agent: {subject: agent}`, the row holding `agent` absent)
  passes the interpreted target under `null_when_absent`, under `omitted_when_absent` and with no
  presence declared.
- A specification that declares no `presence:` on an event field produces the interpreted history
  bytes it did before.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/execute.rs
- cited: crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs
- cited: crates/verify/ess-conformance/tests/interpreted_command_execution.rs
- cited: docs/design/wire-presence-json-prefix.md

Rewritten at the turn-2 close from the merged change (`15647d1b24`); shared surfaces (CHANGELOG.md, the byte-pin tables, generated references) are left out.

## Fit review

1. **The need.** The reference target publishes an absent event field in a spelling the
   specification forbids for that field. Reproduction above; command output in the triage
   directory's `repro/run.txt` (`spec-input-null_when_absent`, `spec-null_when_absent`). The
   requester's proposal, labelled as theirs: "the interpreter carries it as `null`", for one field
   read from the subject; they also report that a declared presence changes nothing.
2. **Class: defect.** `presence: null_when_absent` on an event field means the key is always
   present and `null` spells absence (`docs/design/omitted-or-null-presence.md:20-21`;
   `docs/design/wire-presence-json-prefix.md`, "Conformance": `admits(None)` is refused under
   `null_when_absent`). The interpreter's `emit` adds a field only when a value was determined and
   never reads the field's presence
   (`crates/verify/ess-conformance/src/interpret/execute.rs:3140-3142`).
3. **Already expressible?** Yes, the author already wrote it: `presence: null_when_absent`
   validates (`ess specify validate --path spec-null_when_absent` prints `valid`). Only the
   interpreted target ignores it.
4. **Fit.** No grid cell moves; no authored surface. Siblings inside the interpreter already
   honour the policy: a view row (`crates/verify/ess-conformance/src/interpret/views.rs:451-454`),
   a protected value (`crates/verify/ess-conformance/src/interpret/protected.rs:153-154`) and a
   response field (`crates/verify/ess-conformance/src/interpret/response.rs:470-472`). The event is
   the one left out. The rule applies to the declaring field itself, top level, as the leaf carries
   the policy "only on the leaf that is the declaring field itself and under no other `Optional`"
   (`docs/design/wire-presence-json-prefix.md`, "Conformance"); a struct member's policy inside a
   copied struct is already spelled by the witness. An error's fields are compared with `null` and
   absent as one value (`CHANGELOG.md:498`), so the interpreter's error payload needs no change;
   whether a served error under `null_when_absent` should be held to its spelling: I don't know,
   and it is not this issue. Other targets (generated Rust, Go, TypeScript, web, entity runtime,
   `ess verify diff`, generated docs) are untouched.
5. **A second adopter.** An `InvoiceIssued.due_date: Optional<Date>` declared `null_when_absent`
   because a consumer's schema requires the key; an invoice issued without a due date must run
   green against the interpreted target.
6. **Cost.** No format, keyword or diagnostic. The interpreted history changes only for an event
   field that declares `null_when_absent` and is absent, a case that fails its own suite today.
7. **Alternatives.** (a) Change nothing: every specification with an absent `null_when_absent`
   event field fails `--target interpreted`. (b) Publish `null` for every absent `Optional` event
   field: fails every `omitted_when_absent` leaf. (c) Drop the presence from the leaf when the
   target is interpreted: weakens the suite for every target. Chosen: `emit` writes `null` for an
   absent field declaring `null_when_absent` and leaves it out otherwise.

## Decisions

accept, redesigned. The requester asked for `null` on one subject read; the change follows the
field's declared presence for every absent event field the interpreter publishes, whatever its
source, and keeps leaving the field out where no presence or `omitted_when_absent` is declared.

## Source

github:beyond10x/ess#765

---
format: aep.planning-md/3
id: story:an-external-refusal-before-a-related-row-is-witnessed-at-their-overlap
kind: story
status: draft
title: An external refusal answered before the row is witnessed beside a related-row refusal
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-related-row-may-answer-before-input-guards
revision: 2
---
## Need

A service refuses an input it cannot parse (a cause outside the specification) before it looks for
a stored row with the same name, which it also refuses. Declared in that order, `external:` above a
`when_related:` refusal is refused, because a plain `external:` answers at step 5 and a related row
at step 4. The spelling that states the order exists, `before_row: true` on the external refusal,
which places it at step 1 (`docs/design/generic-language-features.md:491`); but nobody is told so,
and the suite never sends the point where the two disagree.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/c-external-before-row`, `repro/c-idiom`
of the triage of beyond10x/ess#554):

```yaml
outcomes:
  - name: invalid-zone
    external: the zone id does not parse
    error: demo.jobs.InvalidZone
  - name: duplicate-name
    when_related: {entity: demo.jobs.Job, where: name == input.name, exists: true}
    error: demo.jobs.Duplicate
  - name: stored
    creates: demo.jobs.Job
    instance: name
```

```text
$ ess specify validate --path c-external-before-row
error[ESS-COMMAND-004]: `duplicate-name` answers at step 4 of the precedence order and `invalid-zone`, declared above it, at step 5; where both hold, the declaration order and the precedence order disagree
help: declare `duplicate-name` before `invalid-zone`: step 4 answers first in either order
exit 1
```

With `before_row: true` on `invalid-zone` the model validates, synthesizes 3 scenarios and passes
3/3 on the interpreted target; but `invalid-zone`'s scenario forces the provider for a name no row
carries, so a target that checks for a duplicate first passes too. The help names only the
reordering, which changes the specified answer where both hold.

## Acceptance

- `ess verify conform synthesize --path c-idiom` writes a scenario that stores a job, forces the
  external cause, sends the stored job's name, and expects `invalid-zone` with no event and the job
  unchanged; `ess verify conform run --target interpreted` passes it, and an interpreted target
  that reads the related row first fails it.
- A model whose `before_row: true` external refusal has no related-row refusal beside it keeps its
  suite bytes.
- `ess specify validate --path c-external-before-row` exits 1 with the same message, and where the
  earlier-declared branch is a plain `external:` refusal its `help:` also names
  "or mark `invalid-zone` `before_row: true`: it then answers from the request alone, at step 1".

Pinned by `crates/verify/ess-conformance/tests/explore_before_row.rs`
(`an_external_refusal_before_the_row_is_sent_beside_a_present_related_row`,
`no_overlap_scenario_without_a_related_row_refusal`) and
`crates/specify/ess-domain/tests/precedence_classification.rs`
(`an_external_ordering_refusal_names_before_row`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/existence.rs (`externals_on_a_stored_row`, the same witness beside `existing_instance:`)
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- cited: crates/specify/ess-domain/src/command.rs (ordering refusal and its hint, around line 4018)
- cited: crates/verify/ess-conformance/tests/explore_before_row.rs
- cited: crates/specify/ess-domain/tests/precedence_classification.rs
- inferred: docs/design/cross-record-and-stored-field-guards.md (the `before_row` section)
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** An external refusal decided from the request alone must be stated to answer before a
   refusal over a stored row, and the suite must hold a target to that order where both hold.
   Reproduction above. Requester's proposal (theirs, issue comment of 2026-11-06, ess 0.85.0): "a
   declared precedence that lets an `external:` branch answer before a related row".
2. **Class.** Convenience in the spelling, gap in the check. `before_row: true` already states
   the order (`generic-language-features.md:491`, step 1 "`external:` + `before_row: true`, also
   beside `when_related:`"; run.txt § C, `c-idiom` exit 0); the overlap is not witnessed (run.txt
   § C, the `invalid-zone` steps arrange no stored job).
3. **Already expressible?** Yes: `before_row: true` (ess/25, `CHANGELOG.md` "`before_row: true` on
   an `external:` refusal"). The idiom keeps the figure in the specification; this story makes the
   suite check it at the overlap, as `externals_on_a_stored_row` already does beside
   `existing_instance:` (`crates/verify/ess-conformance/src/synthesize/existence.rs:2241-2300`),
   so policy 1 holds.
4. **Fit.** No grid cell opens: `before_row` is an existing key on an existing branch kind.
   Composes with `when_related:` by the rule's table (`generic-language-features.md:491`). The
   witness generalises the stored-row witness from the addressed row to a related row, a sibling
   construct (same question, same answer). Targets: the interpreter already honours it (run.txt
   § C, 3/3); generated Rust and Go keep row-set commands an obligation; Entity Runtime refuses
   `when_related:` by name (`generic-language-features.md:566-584`).
5. **Second adopter.** A booking service that rejects a malformed time-zone or currency code
   (parsed by a library) before it refuses a duplicate booking reference.
6. **Cost.** No source format, no key, no IR change, no diagnostic code; hint text moves (typed
   diagnostic snapshots). Suites of models with the pair grow by one scenario each; no suite
   format moves.
7. **Alternatives.** (a) Change nothing: the adopter reorders and the specification names the
   wrong error where the two errors differ. (b) A new ordering key for `external:`: duplicates
   `before_row`; refused. (c) Move `external:` before step 4 for every command: changes the
   answer of every validating command; refused. (d) Chosen: `before_row: true` is the spelling,
   the hint names it, synthesis witnesses the overlap.

## Decisions

**Accept, redesigned.** No new syntax: `before_row: true` on the external refusal states the
order. Synthesis sends each `before_row: true` external refusal again beside a present related row
a sibling `when_related:` refusal selects, the provider forced, expecting the external refusal and
the row unchanged. ESS-COMMAND-004's hint names `before_row: true` when the earlier-declared branch
is a plain external refusal. No source format, no IR change, no suite format. What changed from the
request: no new precedence declaration; the existing key is the answer. Reply: "Mark
`invalid-zone` `before_row: true`: it then answers from the request alone, before any row is read,
and the suite sends a stored name with an unparseable zone and expects `invalid-zone`."

## Source

github:beyond10x/ess#554

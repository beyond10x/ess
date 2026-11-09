---
format: aep.planning-md/3
id: story:navigation-reaches-any-depth-in-guards
kind: story
status: draft
title: A when_related guard reads a row any number of references away
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:navigation-reaches-any-depth-in-view-filters-and-guards
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 4
---
## Need

A command refuses on a row two declared references away from the row its input names: a note on a
window is refused while the window's incident is still open (port → node → board in the downstream
finding 10 attached to this issue). The `when_related:` guard reads only the row its `via:` names;
a list of steps is a parse error.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/f-guard-two-hops` of the triage of
beyond10x/ess#564):

```yaml
  - name: demo.ops.Annotate
    input:
      - {name: window_id, type: demo.ops.WindowId}
    outcomes:
      - name: no-window
        when_related: {via: input.window_id, exists: false}
        error: demo.ops.NotFound
      - name: incident-open
        when_related: {via: [input.window_id, incident_id], predicate: state == Open}
        error: demo.ops.WrongState
      - name: annotated
        accepts: nothing
```

```text
$ ess specify validate --path f-guard-two-hops
system.yaml: invalid type: sequence, expected a string
exit 1
```

## Acceptance

At G2's next source format:

- `ess specify validate --path f-guard-two-hops` exits 0; `exists: false` over a chain answers a
  missing row at any step; a step that is no declared reference is refused as ESS-COMMAND-003
  naming it;
- the precedence rule places a chained guard at step 4 with every related-row guard
  (`docs/design/generic-language-features.md:494`);
- `ess verify conform synthesize` witnesses `incident-open` with the far row `Open` and `annotated`
  with it `Resolved`, a decoy far row at each step, and a missing middle row answering the
  `exists: false` branch; `ess verify conform run --target interpreted` passes all;
- below that format `via: [..]` in `when_related:` is a named `unsupported_format_version` naming
  the format, not a serde error; a model without a chain keeps its IR bytes;
- generated Rust and Go keep the command an obligation naming the chain; Entity Runtime keeps
  `RelatedGuardUnsupported` (`generic-language-features.md:566-584`).

## Scope

- cited: crates/specify/ess-domain/src/command/related_guard.rs (`via:` parsing and resolution)
- inferred: crates/specify/ess-domain/src/command.rs (value-source chain reused, around lines 1162-1186)
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related.rs
- inferred: crates/specify/ess-domain/src/system.rs
- inferred: schemas/generated/ess.schema.json
- inferred: website/docs/reference/spec-versions.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A refusal decided by the state or a field of a row two references from the input.
   Reproduction above (run.txt § F). Proposed shape (downstream finding 10, 2026-10-19, attached to
   this issue): a guard on a port reads the state of the board that owns the port's node.
2. **Class.** Gap, decided by the design (depth is any chain, answer 8,
   `generic-language-features.md:725`); the one-hop limit is the first cut's (`:248-252`).
3. **Already expressible?** Only by copying the far row's state into the near row at write time,
   which goes stale when the far row moves; no idiom keeps the guard checked against the current
   row.
4. **Fit.** The same `via:` list as the value source and as views
   (`story:navigation-reaches-any-depth-in-view-filters-and-guards`); grid R row `when_related:`
   predicate × related-row column (`docs/design/language-grids.md` Target: every command position).
   Composes with the precedence rule unchanged (step 4). Targets: interpreter and synthesis; Rust
   and Go obligations, named; Entity Runtime refusal kept by decision.
5. **Second adopter.** Booking a seat refused while the seat's room's building is closed.
6. **Cost.** G2's next source format, shared with the view story; no key, no diagnostic code.
7. **Alternatives.** (a) Change nothing. (b) A separate guard per hop: cannot say "the far row of
   this row". (c) Chosen: a `via:` chain, as in values and views.

## Decisions

**Accept as proposed** (the downstream shape, a guard reading the far row's
state, is the design's). `when_related: {via: [input.<id>, <ref>, …], exists | predicate}` from
G2's next source format.

## Source

github:beyond10x/ess#564 (downstream finding 10)

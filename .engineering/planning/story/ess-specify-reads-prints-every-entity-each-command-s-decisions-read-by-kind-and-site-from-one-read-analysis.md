---
format: aep.planning-md/3
id: story:ess-specify-reads-prints-every-entity-each-command-s-decisions-read-by-kind-and-site-from-one-read-analysis
kind: story
status: draft
title: '`ess specify reads` prints every entity each command''s decisions read, by kind and site, from one read analysis the validator can use'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

"Advisory records never decide an approval": no command outside a stated few reads the rows of an
entity in a guard, a selection or a value. A downstream application states this only by
construction: the approving command takes no advisory input, and no command reads the advisory
entity. Nothing checks it, so the next command that gains such a read breaks the claim silently.
Report item 16 asks for a declared claim. A claim needs the set of entities each command reads,
and today no part of ESS computes or prints that set:

- the semantic dependency graph has no read relation from an outcome to an entity its guards or
  values read (`crates/specify/ess-compiler/src/graph.rs:74-130`). It leaves predicate fact paths
  out on purpose (`graph.rs:36-44`);
- `ess specify graph` draws actors, commands and events, with grant, emission and binding edges
  (`crates/generate/ess-gen/src/graph.rs:118-123`);
- the `specify` verbs are `validate`, `compile`, `compose`, `inspect`, `graph`, `realization`,
  `transport`, `runtime` and `toolchain` (`crates/edge/ess-cli/src/main.rs:1755-1785`). `inspect`
  prints one resolved declaration as it is stored.

The model does hold every read, as named constructs: an identity-addressed `when_related` (`via`),
a row-set `when_related` (`entity:`), a rows quantifier `{in: {entity}}`
(`crates/specify/ess-domain/src/command/related_rows.rs:1-17`), a value `{related: …}` (one hop or
two), an `affects:` or `instances:` selection, and the subject an outcome addresses. A binding
carries an event of one command into another command's input.

## Outcome

One function in `ess-domain` lists, for every command, every entity a decision of its outcomes
reads, each with a kind and a site. `ess specify reads --path <dir> [--command <name>] [--entity
<name>]` prints it as text, one line per read, sorted by command, entity and site:

```text
demo.approval.Approve  reads  demo.review.Note  row-set   outcomes.flagged.when_related
demo.approval.Approve  reads  demo.review.Note  binding   bindings.note-to-approval
```

The kinds are `subject`, `related`, `row-set`, `rows-quantifier`, `value`, `selection` and
`binding`. `binding` is a binding that reacts to an event emitted by an outcome acting on the
entity and invokes the command. `g16-entity-readers` calls the same function.

## Acceptance

- Red first: `crates/edge/ess-cli/tests/specify_reads.rs` runs `ess specify reads` on a fixture
  with one command per kind and asserts the exact lines and exit 0. Today the subcommand is
  unknown (exit 2).
- `--command` and `--entity` filter the lines. A name that is no command or entity is refused
  with exit 1, naming it. An invalid specification prints its diagnostics and exits 1, as
  `validate` does.
- The analysis matches exhaustively, with no `_` arm, on every `OutcomeCondition` variant
  (`crates/specify/ess-domain/src/command.rs:590`) and every `PayloadSource` variant (`:1168`). A
  test fails when a variant is added unclassified, so a new read kind cannot go missing silently.
- Two hops of `{related: …}` print one line per hop's entity. A rows quantifier prints its own
  entity beside the selector's.
- The analysis adds no edge to `SemanticDependencyGraph`. An edge there would move the slice
  digest stamped on every generated artifact. Whether impact analysis should reach a command
  through its reads is reported as a separate finding, not changed here.
- The output is text only, and no persisted format is added. The CLI reference lists the verb.
- `examples/billing` prints its reads, and the test pins the line count.

## Fit review

Tree `bcfc51555`. Received 2026-11-24 from a downstream application, report item 16, first half.

1. **Need.** Know which entities each command's decisions read, so that a claim over them can be
   checked. No syntax was proposed for this half.
2. **Class.** *Gap* in tooling: the reads are in the model, and nothing reports them.
3. **Already expressible?** No. `inspect` prints one declaration raw, and `graph` has no read
   edge (citations above).
4. **Fit.** No source surface. A `specify` verb at the second level, as `AGENTS.md` § Agent plugin
   requires ("the first level of `ess` is the five areas and nothing else"). The analysis lives in
   `ess-domain` beside the guards it reads, not in the dependency graph, whose digests it would
   move.
5. **Second adopter.** A personal-data vault whose rows only an export command may read. An
   audit log no command may read.
6. **Cost.** One verb, one function, one test file. No format, IR or diagnostic change. The
   agent plugin's command check (`agentplugins-check tools`) does not cover a new verb until the
   plugin names it. Size M.
7. **Considered.** (a) Read edges in `SemanticDependencyGraph`: moves slice digests and impact
   output. (b) Read edges in `ess specify graph`: a drawing, not a list a test or a claim reads.
   (c) Chosen: a function in `ess-domain` and a text verb.

## Decisions

Part of `P2B-D3` (a) and (c). Open questions:

- **D1 (open, with the owner as AQ2). Do binding flows count as reads?** (a) Yes, as kind `binding`. (b) No. **Recommended:
  (a).** The binding is in the model and carries the entity's events into the command's input.
- **D2 (decided: a, 2026-11-24). JSON output?** (a) Not now: text only. (b) A versioned `--format json`. **Recommended:
  (a).** (b) is a new published format with a schema and a golden file, worth it only when a tool
  asks for it. It becomes needed if the operator takes `P2B-D3` (c).

## Scope

- inferred: `crates/specify/ess-domain/src/command/reads.rs` (new)
- cited: `crates/specify/ess-domain/src/command.rs` (`OutcomeCondition` 590, `PayloadSource` 1168)
- cited: `crates/edge/ess-cli/src/main.rs` (`SpecifyCommand` 149, `specify_area` 1755)
- inferred: `crates/edge/ess-cli/tests/specify_reads.rs` (new), a fixture beside it
- inferred: the CLI reference page under `website/docs/reference/`
- Size: M (about 450 changed lines).

## Depends on

None.

## Source

A downstream application, 2026-11-24, report item 16.

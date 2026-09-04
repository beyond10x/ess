---
format: aep.planning-md/1
id: story:command-line-surface
kind: story
status: implemented
title: A component declares a command-line surface, and ESS synthesizes its parser
relations:
- decomposes: epic:command-line-surface
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/generate/ess-synth/src/clap
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 12
---
# Story: a component declares a command-line surface, and ESS synthesizes its parser

## Defect

`Reach` had two answers to *where are this component's callers* — `in_process` and `network` — and
neither describes a binary a person types at. A command-line caller is deployed with the binary and
is not a program, so `in_process` was nearly right and said the surface never leaves the process;
the surface does leave it, as a **grammar** rather than a call. Nothing in the model could state
that, so a command line was the one surface with no contract ESS could project, and the only
front end that stayed hand-written while `rust`, `go` and `web` were all generated.

## Shape

- `Reach::CommandLine`, stating the same kind of fact the other two state and naming no wire, port,
  path or verb. Which contract follows is derived, as it already is for `network`.
- A `cli:` block on the component: the binary's word, the commands and views at the top level, and
  the groups. Paths within a group derive from `naming.wire`; grouping is declared, because which
  activity a command belongs to cannot be derived from anything the model holds.
- `ess generate synthesize --target clap`, the fourth target beside `rust`, `go` and `web`.

## Refusals

Eight, and they are why the tree is declared here rather than written beside a parser:

| refused | code |
|---|---|
| `cli:` on a component reached another way | `conflicting_declaration` |
| `command_line` with no `cli:` block | `missing_declaration` |
| a placed command the component does not accept | `undeclared_reference` |
| a placed view no domain it owns projects | `undeclared_reference` |
| an accepted command the tree places nowhere | `missing_declaration` |
| a projected view the tree places nowhere | `missing_declaration` |
| a command placed twice | `duplicate_declaration` |
| a view placed twice | `duplicate_declaration` |

The view half comes from a defect found in a consuming repository, not imagined here: `connectors`
serves `kubernetes.workloads` from its personal-local daemon and has no command-line verb that reads
a datasource at all, so an operator cannot reach a projection the process beside them publishes.

## Format

`ess/1`, on the precedent `reached_by` set. Both additions serialise out when unset, so every
existing document digests exactly as before; the published schema grows by 93 lines and loses
nothing. An old-reader test holds that, and the whole committed generated tree is unmoved.

## Acceptance

- `ess validate` accepts a `command_line` component with a `cli:` block, and each of the eight
  refusals has a test that names it.
- `ess generate synthesize --target clap` run twice produces identical bytes.
- The emitted crate compiles with no warnings, its binary is named as the declaration names it, and
  an enum-typed field completes its whole closed set in a generated shell script.
- `task check` exits 0.

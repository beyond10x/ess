---
format: aep.planning-md/3
id: epic:typed-open-questions
kind: epic
status: draft
title: Open questions are typed declarations, refused by default
summary: An authored specification declares what is unsettled as a typed open entry; validate refuses it unless drafting is switched on.
relations:
- serves: vision:O2
revision: 1
---
## Outcome

An open question in a hand-written or agent-drafted specification is a typed declaration ESS reads,
not a YAML comment it never sees. `ess specify validate` refuses a specification that carries one,
by default; a draft that is still being decided opts in explicitly.

## Why

ESS already tracks gaps on its import paths: `ess infra import` reports coverage, diagnostics and
unresolved references (`README.md:94-95`), and the OpenAPI import keeps unpreserved semantics as
durable gaps or refusals and refuses partial input on `--ir` projection (`README.md:102-104`). An
authored specification has no equivalent. The convention for a question nobody has settled is a
`# UNMAPPED:` comment — written by the `ess:specifying` and `aep:planning` skills and emitted by
`aep plan reverse openapi` (`aep/crates/edge/aep-cli/src/reverse.rs:1514-1618`) — and a comment
never reaches the parser, so a specification full of open questions validates. Observed on
2026-10-04: beyond10x/commission and beyond10x/loom validated with 2 and 3 such markers.

The operator's rule of 2026-10-04: the specifications of commission, loom and canon must fully
validate with no open question, as a hard gate; and refusing open questions is ESS's default.

## Scope

1. **Authored gap declaration.** A typed `open:` entry (name to be settled in the first story) that
   names the construct it stands for — an entity, a relation, a field type, a lifecycle, a command
   input — and the question, at the position where the declaration would go. It is carried into the
   compiled IR and listed by `ess specify validate` and `ess specify inspect`.
2. **Refused by default.** `ess specify validate` exits non-zero and names every open entry. A
   specification opts into drafting with an explicit switch (for example `--allow-open`, or a
   `draft: true` in `ess-inputs.yaml` that `--strict` overrides); every projection and
   `ess verify conform synthesize` refuse a specification with open entries regardless.
3. **One vocabulary with the import paths.** The authored entry and the import gaps share one IR
   representation, so `inspect` reports both alike.
4. **Callers move.** `aep plan reverse openapi` emits `open:` entries instead of comments, and the
   `ess:specifying` and `aep:planning` skills describe the entry instead of the comment (changes in
   beyond10x/aep and beyond10x/agentplugins, each through its own process).

## Acceptance

Named scenario `ESS-OPEN-001` passes: a specification with one `open:` entry for an unsettled
relation fails `ess specify validate` by default, naming the entry and its question; passes with the
drafting switch; and is refused by `ess generate synthesize` and `ess verify conform synthesize`
even with the switch. A specification with no open entry validates unchanged.

## Not in scope

Inferring open questions from free text, or treating any comment as meaningful.

## Source

Operator instruction, 2026-10-04 (session planning the Governed Autonomy north star; Atlas
ADR 0066). Interim enforcement in commission, loom and canon is a Rust test per repository that
fails on any `UNMAPPED:` string under `ess/`; it is removed when this epic ships.

---
format: aep.planning-md/3
id: review-result:adversary-w2-exp1-pass-1
kind: review-result
status: active
title: 'W2 EXP1 adversary pass 1: explorer held state, direct responses and component scope'
summary: CONFIRMED, 277 to 279 executed, 1 red (out of scope); F1 owns-filter pin kept, F2 to backlog
relations:
- reviews: story:explorer-and-interpreter-agree-on-external-cause-for-stored-record
- reviews: story:go-explorer-compares-direct-response-members
- reviews: story:unknown-instance-and-input-guard-agree-on-order
- reviews: story:component-explorer-walks-only-its-component
revision: 1
---
unit: W2 EXP1 (story:explorer-and-interpreter-agree-on-external-cause-for-stored-record, story:go-explorer-compares-direct-response-members, story:unknown-instance-and-input-guard-agree-on-order, story:component-explorer-walks-only-its-component), ab6d56ac01 and 15ce4749c6 merged as 3cc17ed596, reviewed in tree ess-w2-exp1 at 15ce4749c6
verdict: CONFIRMED (2 warnings, no blocker)
cases: executed 277→279, red 1
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory exp1/adversary-scratch (logs, compiled IR, model copies)
needs-coordinator: none

**F1 (introduced, warning, fixed).** No unit test read the `owns` half of the component filter
(`crates/verify/ess-conformance/src/go/explore.go:332`) or its view filter (`:339`, and the
TypeScript twin in `scope()`): a mutant dropping either stayed green, because
`explore-component.yaml` declares no `owns:` and no view. The adversary's case
`a_component_owning_a_domain_explores_its_commands_and_views_and_no_others` (two components each
owning one domain) kills both mutants and is kept in
`tests/adversary_w2_exp1_adversary.rs`.

**F2 (pre-existing, warning, out of scope).** A required direct-response member answered as JSON
`null` passes the Go and TypeScript explorers (`explore.go:2690`, `explore.ts:2314`) although the
suite refuses it as `invalid_input` (`src/go/runtime.go:8738`). The story's acceptance covers a
dropped member; at the base the explorers read no response at all. Filed as the backlog draft
story:a-required-response-member-answered-as-null-fails-exploration; the red case is not committed.

**Attacked, not broken.** Held state before an arranged external against `select_phases` (wrong
state, default and `otherwise` answers still offer the external; a `before_row` external is still
offered); Go and TypeScript `decideRow` and `scope` read alike; the component scope matches
`synthesize::handles` and `owns_view`, the `ir.json` digest is checked before scoping in both
languages and an unknown component is refused; dropped, absent, `Optional` and newtype-over-Optional
response members; story 3's pins match the D410 ruling. Peak target/ 6G, lowest / 139G.

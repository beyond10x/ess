---
format: aep.planning-md/3
id: story:a-conformance-run-is-driven-by-the-roles-of-a-composition
kind: story
status: draft
title: A conformance run is driven by the roles of a composition and takes its name mapping from them
summary: A role's commands, views and events map the block's names onto the consumer's; the block's suite runs for what the role covers and reports the rest as not selected
tags:
- adopter-report
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- derived_from: story:a-component-plays-a-pinned-contract-role
- informed_by: story:a-conformance-run-drives-a-target-process-in-any-language
- informed_by: story:role-input-drift-tells-a-caller-from-an-implementation
revision: 1
---
## Need

A consumer system declares in an `ess-composition/4` (or later) document that one of its components plays a component of another released system, a block, through `roles:`. `ess specify compose` checks the claim statically: the command ends exist on their selected surfaces and, for `stands: implements`, the inputs compare (`docs/design/composition-contract-roles.md`, checks 3 and 4). It never runs anything, so a role is proved only by hand today:

1. synthesize the block's suite;
2. narrow it with `ess verify conform select --ids` (suite/5 or later; `crates/verify/ess-conformance/src/coverage.rs:964`, `crates/edge/ess-cli/src/coverage.rs:148`);
3. run it with `ess verify conform run --target process -- <adapter>`, through an adapter written by hand that maps the block's command and view names onto the consumer's (`docs/design/conformance-target-process.md`).

The ids in step 2 and the mapping in step 3 repeat what `roles:` already says, and nothing keeps them in step with it. The design page also leaves two things out on purpose: outputs ("An `implements` entry compares inputs. Outcomes, emitted events and errors are not compared") and a transport-level role. Today a `roles:` entry has no way to name a view or an event at all (`commands` is its only list).

The requester's proposal, labelled theirs: a conformance run driven by `roles`, which runs the block's suite against the consumer's target for the role's commands and views, takes the name mapping from `roles`, and needs no hand-written adapter. Their two acceptance points: outputs, events and views of a role are compared, not only command inputs; a scenario the role does not cover is reported as not selected, never as passed.

## Outcome

`ess verify conform run` can be given a composition and a role. It selects the scenarios of the block's suite that the role covers, runs them against the consumer's target with the block's command, view and event names mapped onto the consumer's by the role, and reports every scenario the role does not cover as not selected. No hand-written adapter maps names, and no hand-kept id list narrows the suite.

## Acceptance

1. A role-driven run of the block's suite against a consumer target passes where the consumer implements the role and fails, naming the scenario and the mapped local name, where it does not. Test: a fixture of a block and a consumer in one new test target of `ess-conformance`, built from a committed suite and a stub target, with no model call and no network, each test under 60 s.
2. Outputs are compared: a consumer whose response, declared error or emitted event differs from the block's in a field the suite checks fails the scenario that checks it, for a command, for an event and for a view read. One failing case for each of the three, named in the test names.
3. A view enters a role: a role lists the views of the consumer that stand for the block's views, and `ess specify compose` refuses an unknown or out-of-component view end with the codes it uses for command ends (`unresolved_semantic_reference`, `reference_outside_component`), with a detail starting `roles[i].views[j]`.
4. A scenario of the block's suite that the role does not cover (it reaches a command, view or event with no role entry, or a `gates` entry) is listed as not selected, with the role named, and is never counted as passed. Test: the report's passed count over a role that covers two of five scenarios is at most two, and the other three are in the inventory's outside list.
5. The same run with the role's entries removed one at a time moves exactly the scenarios that reach the removed entry from selected to not selected (a mutation of the fixture's composition, checked by test).
6. A role-driven run writes the same report/2 a hand-narrowed run writes for the same ids: byte-equal on the fixture, so the existing readers of a report need no change.
7. Old documents keep their bytes: every `/1` to `/6` composition compiles to the bytes it has today (`composition_3_documents_keep_their_ir_bytes_under_the_4_reader` style pin for the new reader).
8. `ess verify conform run --help`, `website/docs/reference/cli.md` and `website/docs/guides/verify/target-process.md` describe the option; the design page gains the section that states the decisions below.

## Fit review

Written with `.agents/skills/assessing-external-requests/SKILL.md`. The consumer is a downstream specification of reusable building blocks.

1. **The need, apart from the syntax.** A system says its component plays another system's component and wants the claim held by the block's own suite, not by a comment or a hand adapter. The fact it cannot express is "run the block's scenarios for this role, with these names". Reproduction: the minimal `demo` and `wire` systems of `docs/design/composition-contract-roles.md`, whose `demo.gate.Admit` and `wire.server.Subscribe` entry composes today; running `wire`'s suite against `demo` needs an adapter that renames the command and a `select --ids` list written beside it. Theirs: a run "driven by `roles`" with a name mapping taken from `roles`.
2. **Class: gap.** The run cannot be expressed from the document at all; a hand adapter and an id list reproduce it outside the specification, which `docs/design/generic-language-features.md` § Policy, policy 1 rules out as an answer.
3. **Already expressible?** Only by hand: `conform select --ids` plus a process-target adapter. That moves the name mapping and the covered set out of the composition into code the specification does not check. Not an idiom for the purposes of question 3.
4. **Fit.** Grid R, what can be read where: a role's ends today are commands only; the design page's own gap is that views, events and outputs have no end. Opening "a role names the other declarations a component answers with" is a whole row (views, events), not one cell, so views and events enter together (decision D2). Reused vocabulary: `local:`, `contract:`, `stands:` keep their meaning; a view or event entry is spelled like a command entry. Siblings: `conformances:` already compares event, error and command payloads as anonymous structs (`ess-composition/5`, `crates/specify/ess-composition/src/conformance.rs`); the output comparison reuses that comparison rather than a second one. Targets: the interpreter, the generators and `ess verify diff` never read a composition (`docs/design/composition-type-conformance.md`), so the run is the only new reader besides `compose`; both are named in Scope. A target that ignores a role entry silently is the red flag the skill names: a run over a role with an entry it cannot map refuses by name.
5. **A second adopter.** A service implementing a published protocol and wanting the protocol's own suite run for its role, such as the OAuth specifications of beyond10x/ess#493 (the same shape the roles story names). Plausible, not observed.
6. **Cost.** A new composition format `/7` for views and events (D2), new composition diagnostics for the new ends, one new option on `run`, no suite or report format change if D4 and D5 are taken as recommended, and no change to `ess/N`.
7. **Considered.** (a) Change nothing: keep the hand adapter and the id list. Rejected: the mapping is the one thing `roles` already states. (b) The requester's shape, a run driven by `roles` with no format change: covers commands only, since views and events have no end to map. Taken in part: the run is taken as proposed, the roles grow ends for what the run must map. (c) A `select` mode that writes a mapped carrier: rejected in D1 because the suite carrier holds the block's names and a mapping is a property of the target the suite runs against.

## Decisions

Accept, redesigned. The requester's wording was taken as evidence of the need. The redesign: the role is the single source of ids and names, the run gains the option, and the roles gain the ends the run needs.

- **D1, where the run lives.** A: `ess verify conform run --composition <file> --role <service>` (the role of the composition whose `service` key is named), which derives the selected ids from the role and wraps the target in a name-mapping layer. B: `ess verify conform select --composition` writes a carrier with mapped names, run afterwards with the existing options. Recommended: A. A carrier holds the block's scenario names and the suite formats do not move; the mapping belongs to the target the suite meets. `ess-composition` is read by `ess-cli` only (`crates/edge/ess-cli/Cargo.toml:28`); `ess-conformance` does not depend on it, so the CLI hands the runner a plain table of mapped names and the dependency stays out of the runner.
- **D2, whether `roles` gains a format `/7`.** A: `/7` adds `views:` and `events:` entries to a role, beside `commands:`, and an output comparison for `stands: implements`. B: no format change, the run covers commands only. C: `/7` adds views only. Recommended: A. Acceptance point 1 needs views and events to have ends, and one source format per design decision, not per request: the entries and the output comparison are one decision. It joins the open composition format if another story opens one before release; the latest shipped is `/6` (`crates/specify/ess-composition/src/lib.rs:84-87`).
- **D3, how a role-mapped name is reported.** A: the report keeps the block's names in scenario ids and check text, and a failure detail carries the local name beside it. B: local names throughout. C: both in every line. Recommended: A. The scenario ids pair the report to the suite's bytes, and a reader sees which block scenario failed first.
- **D4, how an uncovered scenario is reported.** A: it is moved outside the selection with the existing reason `selection_filter` (`coverage.rs:241`), so the carrier's inventory lists it, `passed` never counts it, and the run's summary names the role. B: a new outside reason `not_in_role`, which needs a new suite format. Recommended: A. The inventory already has the account and the strict mode already refuses an incomplete one; a new reason can follow if a reader needs to tell the two apart.
- **D5, what a `gates` entry selects.** A: scenarios that reach a `gates` command are not selected, because the local command decides before the contract command succeeds and does not answer it (`docs/design/composition-contract-roles.md`, the `stands` table). B: run them through the mapping and let them fail. Recommended: A.
- **D6, output comparison.** A: at compose time, for `implements`, every contract output field (response, declared error, event payload) is a local output field of the same name, wire name and conforming type, by the comparison `conformances:` uses, a new `role_output_drift` code; at run time the suite's own checks judge the values. B: run time only. Recommended: A. It keeps the early refusal the inputs already have; the run judges values, which compose cannot.
- Not in this story: a transport-level role (delivering a local event through the contract's delivery) and `stands: calls`, both named as later work by `docs/design/composition-contract-roles.md` and `story:role-input-drift-tells-a-caller-from-an-implementation`.

## Scope

- `crates/specify/ess-composition`: `src/lib.rs` (format constant, `views` and `events` ends of a role, the compiled form), `src/roles.rs` (end admission, output comparison, new codes), `tests/roles.rs`
- `crates/verify/ess-conformance`: the name-mapping target layer over `ConformanceTarget` (`src/target.rs`, `src/process.rs`), the id narrowing through `coverage.rs`, one new test target
- `crates/edge/ess-cli`: `src/main.rs` (`run` option, refusals), `src/coverage.rs`, `src/target_process.rs`
- `docs/design/composition-contract-roles.md`, `docs/design/conformance-target-process.md`
- `website/docs/reference/cli.md`, `website/docs/reference/formats.md`, `website/docs/reference/diagnostics.md`, `website/docs/guides/verify/target-process.md`

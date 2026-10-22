---
format: aep.planning-md/3
id: story:a-row-arranged-beside-is-created-once-under-its-name
kind: story
status: draft
title: A row arranged beside several focus rows is created once under its instance name
summary: With guards on two related inputs, the beside row is re-created under one instance name per focus row and the send names the last copy, so an earlier cross-field branch answers; 2 of 18 downstream scenarios fail on 0.68.0 and the 0.69.0 build.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

A downstream one-domain specification declares `TagMachine` with `when_related` branches on two inputs: `no-machine` (`via: input.machine_id, exists: false`), `no-site` and `frozen` (`via: input.site_id`), then `elsewhere` (`via: input.machine_id, predicate: site_id != input.site_id`), `not-alpha` (`kind != alpha`) and `tagged`. Synthesis writes the scenarios, but `TagMachine/outcome/not-alpha` and `/tagged` fail on the interpreted target with `elsewhere`. In the written suite the site arranged beside the machine rows (beyond10x/ess#283) is opened four times under one instance name, `site-360361`, and each `OpenSite` mints a new identity. Each machine is added to the site the name held at that moment, and the send's `input.site_id` resolves to the last one. The witness machine is therefore in another site, and `elsewhere` answers first.

## Reproduction

`ess verify conform synthesize`, then `ess verify conform run --target interpreted --allow-incomplete --report-format 2` on the downstream specification: 18 scenarios, 15 passed, 3 failed, on 0.67.0 (downstream report), 0.68.0 and the 0.69.0 integration build (commit 52bd110dd).

- `TagMachine/outcome/tagged`: `outcome = elsewhere` where `tagged` was expected; view `repro.fleet.Machines` expected the machine with `site_id …08`, observed it with `site_id …05`.
- `TagMachine/outcome/not-alpha`: `error = repro.fleet.MachineElsewhere`.
- Probe (verified, 0.69.0 build): with the two `input.site_id` branches removed, the command reads one row and every `TagMachine` scenario passes (16 scenarios, 15 passed; the remaining failure is `StartMachine/outcome/frozen`, `story:a-stored-reference-is-pointed-before-the-related-row-moves`).

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs:2138-2185` (`arranged_at` over several rows): the beside rows come from `beside` (`:858`), each at `block_start(base + index, distinction)` (`:875`, `BESIDE = 13`, `:78`). They are bound by `place_beside` (`:2705`) and prepended by `prepend_beside` (`:2720`).
- Inferred, not traced: the focus row and its decoys (`with_row`/`surround`, `:2580-2600`) each carry the beside row's creation steps again under the same instance name, so a later `capture_instance` rebinds it.

Not the cause of `story:an-existence-outcome-passes-a-related-list-membership-guard` (a synthesis-time refusal in the existence family's steered search) nor of a synthesis-time ESS-SYNTH-001: this scenario synthesizes and fails only when run.

## Acceptance (provisional)

- For the specification above, `TagMachine/outcome/not-alpha` and `/outcome/tagged` are synthesized and pass on the interpreted target; each instance name in a scenario is captured once.
- A conformance scenario in this repository holds a minimal specification of this shape (a cross-field `when_related` branch on one input, after `when_related` branches on a second input) to that result.
- A scenario with two `capture_instance` steps of the same instance name fails a suite check at synthesis time.

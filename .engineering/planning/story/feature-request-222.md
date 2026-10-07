---
format: aep.planning-md/3
id: story:feature-request-222
kind: story
status: active
title: validate checks an authored scenario step's expected outcome against the command's guards
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#222
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T12:53:10Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T12:53:10Z", actor: "human:timo", revision: 5}
---
## Outcome

validate checks an authored scenario step's expected outcome against the command's guards (beyond10x/ess#222).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# story:feature-request-222 — fit review (beyond10x/ess#222)

Read tree: `gaps-270`. CLI: `ess 0.44.0`; newer releases may differ.

1. **Need.** `validate` accepts an authored act whose expected outcome the command's own input guards contradict. A correct target fails the scenario, and only a run shows it. Repro `repro-222/` (ess 0.44.0): `scenarios/expired-code-signs-in.yaml` sends `{idp_answer: Code, expiry: Expired}` and expects `signed-in`, while the refusal `expired` (`all: [idp_answer == Code, expiry == Expired]`) claims that input.
   - `ess specify validate --path .`: "demo v1 — 1 file(s), 1 scenario(s), valid".
   - Requester's ask: evaluate the `when:` guards of authored steps at validate time, at least guards that read no subject, and refuse a step whose expected outcome is not the one selected.
2. **Class: gap** (in checking, not in expression).
   - Nothing documents the check: authored validation resolves names, shapes, actors and external answers (`website/docs/reference/diagnostics.md:128-164`, ESS-AUTHOR-001..037; CHANGELOG.md:1328, #112).
   - A partial workaround exists. `ess verify conform run --target interpreted --path spec/signin.yaml --scenarios scenarios` reports the act as `failed`. A consistent control act (`control/fresh-code-signs-in.yaml`) is reported `unsupported`, not passed, so the interpreter's coverage is partial and the result gives no reason.
3. **Already expressible?** Only through that interpreted run (above). It is not part of `validate`, and it is silent where the interpreter does not support the command.
4. **Fit: passes as proposed.** No authored surface is added, only a new diagnostic (`ESS-AUTHOR-038`).
   - Machinery it reuses:
     - the three-valued guard evaluator: `crates/verify/ess-conformance/src/decision.rs:29-41`, `:77-90` ("unevaluable" means stay silent);
     - the decided-branch logic of ESS-AUTHOR-037: `crates/verify/ess-conformance/src/authored.rs:2266-2290`.
   - It must follow the stated precedence (`docs/design/input-guard-overlap-precedence.md`):
     - input-guarded refusals come before accepting branches;
     - declaration order holds among refusals and among accepting branches;
     - the default branch is what no other guard claims.
   - Only `When` / `Otherwise` conditions are decided. `External`, `WrongState`, `SubjectState` (state half), `Related` and `SubjectField` are not input-decidable (`decision.rs:16-28`) and are skipped, not guessed.
   - An `{$instance: …}` input is opaque, so a guard that reads it is skipped.
   - The same check applies to an act's `error:` claim when `outcome:` is omitted.
   - Targets: `validate` / `author` only. Runners and formats are unchanged. The coverage inventories accept the new code, as was done for 037 (CHANGELOG.md:493-494).
5. **Second adopter: yes.** A transfer command with `when: amount <= 0` → `InvalidAmount`, and an authored act sending `amount: 0` that expects `accepted`. This arises wherever authored scenarios sit beside input-guarded refusals.
6. **Cost.**
   - One diagnostic code, docs, and the inventories.
   - No format bump and no diff class.
   - Newly refused: documents whose authored acts contradict their guards. Those already fail against any correct target.
7. **Alternatives.**
   - (a) Change nothing and point to the interpreted run: partial coverage and no reason given.
   - (b) Run authored scenarios through the interpreter inside `validate`: heavy, and it imports the interpreter's `unsupported` gaps into validate.
   - (c) Statically decide input-only guards with the precedence rules, three-valued: chosen. This is the request in substance.

## Decisions

- **accept as proposed (proposed):**
  - **The check:** `validate` / `author` refuse an authored act (new `ESS-AUTHOR-038`) whose literal input, decided under the stated precedence (`input-guard-overlap-precedence.md`), selects a branch other than its `outcome:`, or other than one reporting its `error:`.
  - **Scope:**
    - It decides only `when:` / default conditions.
    - It stays silent where the evaluator answers "unevaluable" or a guard reads an `{$instance}` input, a subject, a related row or an external answer.
    - No format change.
  - **Overlaps, not duplicates:**
    - #112, closed: validate reads authored scenarios.
    - #243 / ESS-AUTHOR-037, closed: same decided-branch machinery.
    - #178 / #217, closed: the precedence it must apply.
    - #280, open: synthesis ignores an enum-and-presence input guard. The same evaluator path, so check it is not inherited.
  - **Already fixed?** No (ess 0.44.0 validates the repro).

- Coordinator (2026-10-01): adopted, with the next free ESS-AUTHOR code (038 to 040 are taken by #265).

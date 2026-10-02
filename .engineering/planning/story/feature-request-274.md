---
format: aep.planning-md/3
id: story:feature-request-274
kind: story
status: implemented
title: A generated CLI maps invalid input to a declared callable error
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#274
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-cli-project
- confidence: cited
  path: crates/specify/ess-cli-contract
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T19:35:48Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-01T19:35:49Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-02T12:27:43Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A generated CLI lets a callable declare invalid_input: <code>, naming one of its declared errors. Invalid typed input and malformed, empty, duplicate-keyed or validator-rejected dynamic input answer that code with empty data and exit 2, without exposing the input text. The DynamicValidator API remains unchanged.

## Acceptance

- Admission accepts a declared error whose type accepts {}; it refuses undeclared codes, incompatible error types, inputless callables and cli_parse.
- With invalid_input configured, unparsable or empty dynamic input, including empty stdin, and duplicate JSON keys answer the declared code, {} data and exit 2. Neither validator nor handler runs.
- Valid JSON rejected by DynamicValidator with Input-phase InvalidValue receives the same answer; the handler does not run.
- Typed input parsing or shape failures receive the same declared-error answer.
- Failure output contains no input text; human output names only the code. Valid dynamic input still reaches validation and handling.
- Without the key, cli_input and cli_dynamic_input remain unchanged. Duplicate keys are rejected at every nesting depth; equal keys in separate objects remain valid.
- The compiled plan preserves the mapping, omits an absent mapping, and generated reference documentation names it.

These criteria reconcile the obsolete initial text with the already-adopted Decisions; they do not adopt a new DynamicValidator API. Existing named cases live in ess-cli-project/tests/invalid_input.rs, ess-cli-contract/tests/binding.rs::invalid_input_names_a_declared_error_that_needs_no_field, and the generated-package process fixture selected by projection.rs. Seven focused tests passed in independent review; full package verification is running before lifecycle closure.

## Origin

beyond10x/ess#274, reported downstream on 0.45.0 and 0.48.0 (`ess-cli-project/src/runtime.rs:666`).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-274 (beyond10x/ess#274)

There is no `story:feature-request-274` in the read tree's store (`aep plan artifact show` errors) or on `origin/main` (`git ls-tree`). The story must be created before scheduling.

1. **Need.** A generated CLI's error contract cannot answer an invalid dynamic payload with one of the callable's own declared errors. An unparsable or empty payload always answers the adapter's closed code `cli_dynamic_input`. So does a payload the adopter's validator rejects: `DynamicError::InvalidValue` → `cli_dynamic_input`, `crates/generate/ess-cli-project/src/runtime.rs:668-669,738-745`. Only a payload the validator accepts reaches the handler, which can answer a declared code. Code reading in the read tree (`e3bc9a2ff`):
   - `runtime.rs:657-667`: `serde_json::from_str` fails → `internal(output, 2, "cli_dynamic_input")` before the validator. Identical at tags 0.44.0 and 0.48.0 (`git grep cli_dynamic_input <tag>`).
   - A secondary finding explains the requester's "duplicate key reaches the validator". `serde_json` builds a `Value` map last-key-wins with no error (`serde_json-1.0.151/src/value/de.rs:136-143`). So the validator checks a value other than the text the handler receives in `invocation.input`. This is inferred from source and was not run.
   - Reproduction (`repro-274/`, ess 0.44.0): `ess specify cli --path model.yaml --binding cli.yaml` → valid (`ok.out`). `ess generate cli … --out gen` emits the same code at `gen/src/runtime.rs:666,669`.
   - Requester's proposal, labelled as theirs: (a) hand unparsable or empty input to the validator as raw text or a typed parse error; or (b) let the binding declare the failure code.
2. **Class.** **Gap.** The documented design (`docs/design/cli-presentation-binding.md:100-108`) says `InvalidValue` refuses native input with exit 2 and never names a code an adopter can choose, so ESS behaves as documented. The binding cannot state the domain fact "invalid input to this callable answers `<declared error>`". The duplicate-key collapse is a **defect** candidate. The design says the runtime must "validate input", and it validates a lossy reading of it.
3. **Already expressible?** No.
   - Partial workaround: a `local` target with the payload as a plain `String` input, where the handler parses and answers the declared code for all three cases.
   - That loses the generated dynamic validation of input, result and error data (`runtime.rs:705-736`), so it is a downgrade, not an idiom.
   - The requester's option (b) shape is refused today: `repro-274/cli-proposed.yaml` → `unknown field invalid_input` (`proposed.out`).
4. **Fit.**
   - Option (a) does **not** meet the need. Even routed through the validator, `InvalidValue` still maps to `cli_dynamic_input` (`runtime.rs:741`). It would only work by widening the closed `DynamicError` to carry a code, which breaks the generated `DynamicValidator` API for every adopter.
   - Option (b), redesigned:
     - The key names a declared error from the callable's existing `errors` map (the same vocabulary).
     - It applies to parse failure, empty input and validator `InvalidValue` on the Input phase alike, so the three cases agree.
     - Admission must require that the error type admits a value with no field present, because the output must carry no input text. ESS-TYPE-007 already refuses a fieldless struct, so every field must be Optional (seen in repro: `model.yaml` with `fields: []` refused).
   - Sibling: typed callables fail shape checks with `cli_input` (`runtime.rs:575`), and the same question arises there. A callable-level key covering both `cli_input` and `cli_dynamic_input` is the general form. `cli_parse` happens before a callable is known (`runtime.rs:599-611`) and is refused by name.
   - Targets: `ess-cli-contract` admission, `ess-cli-plan/1`, the generated runtime, the generated reference/help, and `--check` drift.
5. **Second adopter.** Any generated CLI whose JSON error codes are a published contract for scripts, for example a tool-invocation CLI that promises `bad_request` for every malformed request, whatever the cause.
6. **Cost.**
   - One optional key in `ess-cli/1`. Whether an additive key needs `ess-cli/2`: I don't know; there is no versioning note in `cli-presentation-binding.md`.
   - One new admission diagnostic (the named error is undeclared, or its type requires a field).
   - The generated `runtime.rs` is copied into every package, so every adopter sees a byte diff on regeneration (`--check` fails until regenerated), even without using the key.
   - The `DynamicValidator` API is unchanged.
   - Duplicate-key refusal at parse changes behaviour for documents that rely on last-key-wins (unlikely, but a behaviour change).
7. **Alternatives.**
   - (i) Change nothing. Adopters remap codes outside the CLI or use a `local` target. Rejected: it pushes a contract fact out of the binding.
   - (ii) Requester (a): rejected, because it does not change the answered code without breaking the validator API.
   - (iii) Requester (b) as written (a dynamic-only key holding a free code): changed to name a *declared* error, to cover validator `InvalidValue` too, and to extend to `cli_input`.
   - (iv) Chosen: (iii), plus refusing duplicate keys at parse so the validator sees exactly the text the handler gets.

## Decisions

- **accept, redesigned (proposed):** Add a callable-level key in `ess-cli/1` that names one of the callable's *declared* errors as the answer for invalid input.
  - It covers unparsable and empty dynamic payloads, validator `InvalidValue` on the Input phase, and typed `cli_input` shape failures.
  - Admission refuses an undeclared error, and an error type with a required field, since the output must stay free of input text.
  - `cli_parse` is refused by name, because it fails before a callable is known.
  - Also refuse duplicate JSON keys at parse (serde_json is last-key-wins), so the validator sees exactly what the handler receives.

  The requester's option 1 (route to the validator) was rejected: `InvalidValue` still maps to `cli_dynamic_input` (`runtime.rs:741`) unless the `DynamicValidator` API breaks.

  Not fixed: same code at 0.44.0, 0.48.0 and the read tree (`runtime.rs:666,669`). No duplicate or overlap among open or closed ESS issues (searched "cli_dynamic_input", "dynamic input", "DynamicValidator"). No story artifact exists yet.

- Coordinator (2026-10-01): adopted as proposed above.

## Released redesign located; reconcile acceptance before closure

Release0.51.0 CHANGELOG and current source contain the adopted invalid_input declared-error mapping. crates/generate/ess-cli-project/tests/invalid_input.rs covers unparsable/empty/duplicate/rejected dynamic input, typed shape failures, unchanged legacy codes, duplicate keys at every depth and generated reference text. crates/specify/ess-cli-contract/tests/binding.rs:311 tests declared-error admission, required-field refusal and unchanged no-key plans. The existing Acceptance text still says malformed data reaches DynamicValidator, which the explicit Decisions reject; it must be reconciled to the accepted callable-level declared-error behavior rather than used to demand the rejected API design. This audit locates source/tests but claims no fresh CLI-project test execution or final artifact closure.

## Released behavior fully verified

Independent reviewer server_corrections ran both full affected packages at ba5d657b72913b29ecc19e9a6d2c2a111f083793: ess-cli-project 155 outer passes plus 13 actual generated-package process cases, ess-cli-contract 15 passes; 183 total, zero failures and zero ignored, both commands exit zero. The generated process case declared_invalid_input_answers_unparsable_empty_and_duplicate_payloads executed. Logs retained as target/backlog-input/274-full-project.log and 274-full-contract.log in the combined synthesis worktree, with matching .exit files. The last command completed 2026-10-02T12:25:24Z.

Both package trees are byte-identical to release0.51.0 at0347ffa222939e3791e574d2dbe42d4b4b02d979 and main b4da64e38b770fe74103409fe1fef7ae6ca214f4. The public design and existing tests establish the adopted declared-error redesign; the obsolete outcome/acceptance/title have been reconciled to that recorded decision. This closes stale planning state; no new production implementation is claimed.

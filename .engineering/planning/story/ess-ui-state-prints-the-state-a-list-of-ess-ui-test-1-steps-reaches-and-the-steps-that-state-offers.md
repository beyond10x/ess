---
format: aep.planning-md/3
id: story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers
kind: story
status: draft
title: ess ui state prints the state a list of ess-ui-test/1 steps reaches and the steps that state offers
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
revision: 1
---
## Need

`g01-session-actions` and `g01-state-document` give a Rust program the offered steps and the state.
A program in another language, and a golden test written from the command line, have no route.
ESS's published surface is its command line and its formats; the crates are `publish = false`
(`Cargo.toml:61`).

- `ess ui` has the verbs `load`, `check`, `docs`, `run` and `test`
  (`crates/edge/ess-cli/src/ui.rs:13-28`).
- `ess-ui-test` can read a list of steps (`parse_steps`, `crates/ui/ess-ui-test/src/spec.rs:343`),
  but nothing writes a `Step` back in its `ess-ui-test/1` spelling (`spec.rs` has parsers only,
  `:297`, `:343`). The offered steps cannot be printed as steps.

## Outcome

- `ess ui state --path <document> [--model <spec>] [--tests <file> --test <name>] [--steps <file>]`
  opens the document. It uses the fixtures of the named test of an `ess-ui-test/1` file when one
  is given. It applies the steps of `--steps`, a YAML list in the `ess-ui-test/1` step spelling,
  and prints the `ess-ui-state/1` document of the state it reached, with one more member,
  `offers`.
  - `offers` lists the steps that state offers, each written as an `ess-ui-test/1` step. A step
    that takes a free value is written with its target only (P0-D4).
  - `offers` is outside the digest, which covers `state` only.
- A step that fails prints the message `ess ui test` prints for it, on stderr, and the command
  exits 1. A document that does not load is refused as `ess ui load` refuses it.
- `ess_ui_test::write_step(&Step) -> ess_yaml::Value` writes a step in the spelling `parse_steps`
  reads.

## Acceptance

New test file `crates/edge/ess-cli/tests/ui_state.rs`. Each test is expected under 5 s and is
timed. None may run over 60 s, and the file's step stays within 120 s.

1. `ess_ui_state_prints_the_home_state_and_its_offers`. Red today: `ess ui state` is an unknown
   subcommand.
   - Run `ess ui state --path examples/partner-portal/ui.yaml`.
   - The printed `state` and `digest` equal the golden file of `g01-state-document`.
   - `offers`, read back through `parse_steps`, equals `Session::offered` for that state.
2. `a_step_list_reaches_the_state_the_library_reaches`. With `--steps` holding one `select` of the
   second row, the printed `digest` equals the library's digest after that step.
3. `a_failing_step_names_itself`. With a step at a node path that does not exist, the message is
   the runner's message for it, on stderr, and the command exits 1.
4. `every_step_kind_writes_back_to_itself` (in `crates/ui/ess-ui-test/tests/`). For one example of
   each of the 14 step kinds of `STEPS` (`lib.rs:41-56`), `parse_steps(write_step(step)) == step`.

Also:

- The CLI reference (`website/docs/reference/cli.md`, beside `ess ui load` at `:1855`) lists
  `ess ui state` and its flags. Any xtask pin that counts CLI leaves is moved with this cause.
  `task test-xtask` and `task ci-lint` pass.
- The `ess-ui-state/1` row of `formats.md` names `offers` as written only by `ess ui state` and
  outside the digest. `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1, under P0-D3 (A).

1. **Need.** The same offered steps and state for a program that does not link Rust.
2. **Class.** This is a *gap* in reach: the library has it, the command line does not.
3. **Already expressible?** Partly. A one-test file per step path, run with `ess ui test`, says
   pass or fail and nothing more.
4. **Fit.**
   - It follows `ess ui check --format json` and `ess ui test --format json`: one verb, a JSON
     document with a format id (`formats.md:219-221`).
   - It uses the step spelling of `ess-ui-test/1` for both its input and its `offers`.
   - It replays from `open` on each call, which is what P0-D2 (A) accepts for a process boundary.
5. **Second adopter.** A shell script or a CI job that checks a state digest after a fixed step
   list, to detect a behaviour change.
6. **Cost.**
   - One CLI verb, one optional member of `ess-ui-state/1`, and a step writer.
   - No new format. No diagnostic.
7. **Considered.**
   - (a) No verb (P0-D3 B).
   - (b) A long-running process that reads steps on stdin and keeps the state in memory. This
     adds a protocol for one request.
   - (c) Chosen: one verb that replays a step list.

## Decisions

This story follows P0-D3. Under (B) it is not built, and the rest of item 1 is unaffected.

## Scope

- cited: `crates/edge/ess-cli/src/ui.rs` (`Command` 13-28, `run` 99-; a `State` verb)
- cited: `crates/ui/ess-ui-test/src/spec.rs` (a `write_step` beside `parse_steps` 343),
  `crates/ui/ess-ui-test/src/lib.rs` (export)
- new: `crates/edge/ess-cli/tests/ui_state.rs`; a test in `crates/ui/ess-ui-test/tests/`
- `website/docs/reference/cli.md`, `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: M, about 350 changed lines (step writer about 130, verb about 80, tests about 140).

## Depends on

- `g01-state-document`
- `g01-session-actions`

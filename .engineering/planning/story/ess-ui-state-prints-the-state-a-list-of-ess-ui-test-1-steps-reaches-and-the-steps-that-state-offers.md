---
format: aep.planning-md/3
id: story:ess-ui-state-prints-the-state-a-list-of-ess-ui-test-1-steps-reaches-and-the-steps-that-state-offers
kind: story
status: active
title: ess ui state prints the state a list of ess-ui-test/1 steps reaches and the steps that state offers
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-ui-state-prints-as-ess-ui-state-1-with-a-digest-and-two-routes-to-one-state-print-the-same-bytes-and-the
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
- depends_on: story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:24Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:25Z", actor: "human:timo", revision: 4}
---
## Need

`g01-session-actions` and `g01-state-document` give a Rust program the offered steps and the state.
The requester names a command that replays a fixed step list as an inspection aid, and a golden
test written from the command line needs one too. ESS's published surface is its command line and
its formats; the crates are `publish = false` (`Cargo.toml:61`).

- `ess ui` has the verbs `load`, `check`, `docs`, `run` and `test`
  (`crates/edge/ess-cli/src/ui.rs:13-28`).
- `ess-ui-test` can read a list of steps (`parse_steps`, `crates/ui/ess-ui-test/src/spec.rs:343`),
  but nothing writes a `Step` back in its `ess-ui-test/1` spelling (`spec.rs` has parsers only,
  `:297`, `:343`). So the offered steps cannot be printed as steps.
- A step that takes a free value cannot be spelled by target alone: `parse_steps` requires
  `type: {at, text}` (`spec.rs:542-547`) and an upload's file (`:519`).

The verb is not the search path. The requester's caller links the crates; no long-running process
is wanted (`g01-state-design`, Out of scope).

## Outcome

- `ess ui state --path <document> [--model <spec>] [--tests <file> --test <name>] [--steps <file>]`:
  - It opens the document through `Session::start`, with the model when given.
  - It uses the fixtures of the named test when one is given, and otherwise those the document's
    `fixtures:` index names.
  - It applies the steps of `--steps`, a YAML list in the `ess-ui-test/1` step spelling.
  - It prints the `ess-ui-state/1` document of the state it reached, with the optional member
    `offers`.
- **`offers`** lists the steps the state offers, each in the `ess-ui-test/1` spelling. A step that
  takes a free value is written by target only, as `{type: {at: <path>}}` or
  `{upload: {at: <path>}}` (P0-D4). `offers` is outside the digest, which covers `state` only.
- **The format decision for `offers`.** It is an optional member added to `ess-ui-state/1` in the
  release that publishes the format. It keeps `/1`: a reader of the state document without
  `offers` still reads it, which an old-reader case proves (`AGENTS.md`, determinism and formats).
- A step that fails prints the message `ess ui test` prints for it, on stderr, and the command
  exits 1. A document that does not load is refused as `ess ui load` refuses it.
- `ess_ui_test::write_step(&Step) -> ess_yaml::Value` writes a step in the spelling `parse_steps`
  reads. `ess_ui_test::parse_offers(&ess_yaml::Value) -> Result<Vec<Offered>, TestError>` reads
  `offers` back.

## Acceptance

New test files `crates/edge/ess-cli/tests/ui_state.rs` and `crates/ui/ess-ui-test/tests/step_writer.rs`.
Each test is expected under 5 s and is timed; none may run over 60 s, and each crate's test step
stays within 120 s. Each file joins its crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

1. `ess_ui_state_prints_the_home_state_and_its_offers`. Red today: `ess ui state` is an unknown
   subcommand.
   - Run `ess ui state --path examples/partner-portal/ui.yaml`.
   - The printed `state` and `digest` equal those of the live golden of `g01-state-document`.
   - `offers`, read back through `parse_offers`, equals `Session::offered` for that state.
2. `a_step_list_reaches_the_state_the_library_reaches`. With `--steps` holding one `select` of the
   second row, the printed `digest` equals the library's digest after that step.
3. `a_named_test_supplies_the_fixtures`. With `--tests examples/partner-portal/tests/stale.yaml
   --test <its first test>`, the printed `digest` equals the library's digest for a session
   started with that `Test`. It differs from the run without `--tests`.
4. `a_model_reaches_the_state`. With
   `--path crates/ui/ess-ui-test/tests/fixtures/fixture-command-effect/ui/desk.ui.yaml`, its model
   and a `--steps` list holding one `act`, the printed `model` is `true`. The state equals the
   library's for a session started with that model.
5. `a_failing_step_names_itself`. With a step at a node path that does not exist, the message is
   the runner's message for it, on stderr, and the command exits 1.
6. `the_frozen_state_golden_reads_with_offers` (`ui_state.rs`). The frozen golden of
   `g01-state-document`, with an `offers` member added, still deserialises to the same `state`,
   `digest` and `document`.
7. `every_step_kind_writes_back_to_itself` (`step_writer.rs`). For one example of each of the 14
   step kinds of `STEPS` (`lib.rs:41-56`), `parse_steps(write_step(step)) == step`.
8. `offers_read_back` (`step_writer.rs`). `parse_offers` reads `{type: {at: P}}` as
   `Offered::Type { at: P }` and `{upload: {at: P}}` as `Offered::Upload { at: P }`. It reads every
   other entry as `Offered::Step`.
9. `the_stable_surface_compiles` (`crates/ui/ess-ui-test/tests/session.rs`) adds `write_step` and
   `parse_offers`.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-cli -p ess-ui-test --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-test --locked`;
- `cargo test -p ess-cli --test ui_state --locked`;
- `task test-xtask` and `task ci-lint`;
- `task projection-check` (`cargo xtask cli-reference --check` compares `cli.md`, and
  `cargo xtask docs` checks the formats row).

Also:
- The CLI reference (`website/docs/reference/cli.md`, beside `ess ui load` at `:1855`) lists
  `ess ui state` and its flags. `cli.md` is regenerated by `cargo xtask cli-reference`, never
  edited by hand. Any xtask pin that counts CLI leaves is moved, naming this cause.
- The `ess-ui-state/1` row of `formats.md` names `offers`: optional, written only by
  `ess ui state`, outside the digest, with its spelling for a free-value step.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 1, under P0-D3 (A).

1. **Need.** The requester names `ess ui state --steps` as an inspection aid: one state and its
   offered steps for a fixed step list, from the command line. A golden test from the command
   line needs the same.
2. **Class.** It is a *gap* in reach: the library has it, the command line does not.
3. **Already expressible?** Partly. A one-test file per step path, run with `ess ui test`, says
   pass or fail and nothing more.
4. **Fit.**
   - It follows `ess ui check --format json` and `ess ui test --format json`: one verb, and a JSON
     document with a format id (`formats.md:219-221`).
   - It uses the step spelling of `ess-ui-test/1` for both its input and its `offers`.
   - It replays from `open` on each call. That is fine for an inspection aid, and the search path
     is the library.
5. **Second adopter.** A shell script or a CI job that checks a state digest after a fixed step
   list, to detect a behaviour change.
6. **Cost.**
   - One CLI verb, one optional member of `ess-ui-state/1`, a step writer and an offers reader.
   - No new format and no diagnostic.
7. **Considered.**
   - (a) No verb (P0-D3 B).
   - (b) A long-running process that reads steps on stdin and keeps the state in memory. The
     requester wants none.
   - (c) Chosen: one verb that replays a step list.

## Decisions

This story follows P0-D3. Under (B) it is not built, and the rest of item 1 is unaffected.

## Scope

- `crates/edge/ess-cli/src/ui.rs`: `Command` 13-28, and a `State` verb in `run` 99-.
- `crates/ui/ess-ui-test/src/spec.rs`: `write_step` and `parse_offers` beside `parse_steps` (343).
- `crates/ui/ess-ui-test/src/lib.rs`: exports.
- new: `crates/edge/ess-cli/tests/ui_state.rs`, `crates/ui/ess-ui-test/tests/step_writer.rs`.
- `crates/ui/ess-ui-test/tests/session.rs`: two names added to `the_stable_surface_compiles`.
- `website/docs/reference/cli.md` (generated), `website/docs/reference/formats.md` (the `offers`
  sentence in the row).
- `CHANGELOG.md`.
- Size: M, about 380 changed lines: step writer and offers reader about 150, verb about 80, tests
  about 150.

## Depends on

- `g01-state-document`: the format, its row and its goldens.
- `g01-session-actions`: `Session` and `Offered`.
- `g03-resolved-binding`: `ui.rs` `run` and `cli.md`, in round 7.

Round 8, alone. It ends the wave that publishes `ess-ui-resolved/1` and `ess-ui-state/1`.

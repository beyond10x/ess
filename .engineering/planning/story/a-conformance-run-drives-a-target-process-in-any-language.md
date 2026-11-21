---
format: aep.planning-md/3
id: story:a-conformance-run-drives-a-target-process-in-any-language
kind: story
status: active
title: A conformance run drives a target process written in any language
summary: ess verify conform run starts an outside program and asks it ConformanceTarget's questions over a versioned protocol on stdin/stdout; the Rust runner judges and writes report/2
relations:
- decomposes: epic:adopter-gaps-verdicts
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design
- confidence: inferred
  path: website/docs/guides/verify/runners.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T23:41:09Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-21T02:34:03Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

`ess verify conform run` holds an implementation outside the `ess` binary to a suite. The operator
names a command; `ess` starts it, asks it the questions of `ConformanceTarget` over a documented,
versioned protocol on the process's standard input and output, judges the answers with the Rust
runner, and writes the same report/2 a built-in target gets. An implementation in any language,
a web service or a command-line program, becomes a target by answering that protocol, with no
generated runner and no second toolchain in its tree.

## Need

Request of 2026-11-21 from a downstream prototype in which agents write an implementation from a
specification, in a directory and a language of their choice (the example given: a Python 3 web
service with a JSON API, or a command-line program). Its test step runs
`ess verify conform run --target interpreted`, which checks the model and nothing about the
agent's code, so a story whose implementation is wrong is reported done.

The requester's proposal, theirs and not the design: "a way to declare an outside target (a process
to start, or an endpoint/command adapter that answers ExecuteCommand and QueryView, or a documented
target protocol over stdin/stdout or HTTP) so `ess verify conform run` executes the synthesized
scenarios against it and reports passed/failed/skipped per scenario, deterministically, with a
nonzero exit when any scenario fails."

## Fit review

1. **The need, apart from the proposed syntax.** An implementation that is not Rust, Go or
   TypeScript cannot be the system under test of a run `ess` executes and judges.
   `ess verify conform run --target` takes `billing`, `oracle-fixture` and `interpreted`
   (`crates/edge/ess-cli/src/main.rs:1207-1214`; `ess verify conform run --help` on 0.96.0 prints
   `[possible values: billing, oracle-fixture, interpreted]`), and the guide says so:
   "`ess verify conform run` runs a suite only against the targets built into `ess`"
   (`website/docs/guides/verify/runners.md:47`). Minimal reproduction: `examples/billing`, a suite
   from `ess verify conform synthesize --path examples/billing --out suite.json`, and any program
   outside this workspace that implements billing; no `--target` value reaches it.

2. **Class: gap.** `docs/design/running-a-suite.md:96` lists the out-of-process adapter as "the
   named open gap", and `:106-112` quotes the plan that built the runner: "An out-of-process
   implementation cannot be run. `ConformanceTarget` is a Rust trait, and the binary can only reach
   an implementation it was compiled with." No authored document changes: the request adds no key
   to `ess/N` and no step to a suite.

3. **Can it already be expressed?** In part, three ways, each with a cost this story removes:
   - The generated Go or TypeScript package (`ess verify conform synthesize --target
     go|typescript`, `website/docs/guides/verify/runners.md:47-56`): the implementer writes a
     `Target` in Go or TypeScript that calls the real implementation. Verified 2026-11-21 on
     0.96.0: the TypeScript package for a 32-scenario billing specification is 15 files and runs
     with `ESS_REPORT_FORMAT=2 npm test`. For an implementation in another language this puts Node
     or Go and a second-language adapter into its tree.
   - A runner the adopter writes, then `ess verify conform report --results`
     (`crates/edge/ess-cli/src/main.rs:821`): the adopter's runner judges, and the report says so
     (`producer_profile: external-scenario-status/1`). `docs/design/running-a-suite.md:173-192`
     refuses handing the verdict to a foreign runner as the supported route.
   - The Rust trait (`crates/verify/ess-conformance/src/target.rs:103`): Rust only, and the crate
     is unpublished (`docs/design/running-a-suite.md:95`).
   The idiom condition holds for the first route (the suite still checks every figure), so the
   requester has been given it as the route for today. It does not close the gap for a language
   with no emitter: `story:java-conformance-target` prices one emitter at about 66 KB of
   hand-written runner per language.

4. **Does the design fit what is already there?** The language grids do not apply: no cell of grid
   R, grid O or the effect × target grid is opened, because nothing authored changes.
   - *Vocabulary.* The protocol's operations are the methods of `ConformanceTarget`
     (`target.rs:103-583`), spelled as the trait spells them; its request and answer members are
     the ones the generated TypeScript `Target` already exchanges as plain JSON
     (`crates/verify/ess-conformance/src/ts/`, the `Target` interface of `runtime.ts`). No second
     spelling of a request is introduced. The request types already derive `serde::Serialize`
     (`target.rs:1002-1217`).
   - *The verdict stays with the runner.* The target returns observations; `Runner::run`
     (`crates/verify/ess-conformance/src/runner.rs:377`) judges. This is ADR
     `architecture-decision-record:ess-evolution-09-independent-conformance` ("Targets return
     observations; runners judge them") and `target.rs:63-66`.
   - *Unsupported stays a failed run.* An operation the target does not answer is
     `TargetError::Unsupported` and the scenario is `unsupported`, as `running-a-suite.md:42-58`
     requires. No skip is added.
   - *Exactness.* `target.rs:96-101` requires a JSON adapter to establish exact Integer decoding.
     Here `ess` itself decodes the answer bytes, so the exactness rule is enforced in one Rust
     reader and not left to each adapter.
   - *Siblings.* `ess verify conform mutate --target` shares `ReferenceTarget`
     (`main.rs:4534-4554`); it takes the same outside target or refuses it by name, decided on the
     design page. `mutate --emit/--collect` (`story:mutate-drives-an-external-target`) is
     unchanged. `story:a-conformance-run-drives-a-live-http-target` (draft) stays its own story: a
     target reached through its `ess-http/1` binding has no scenario isolation, event observation
     or setup operation of its own, and can later be built as an adapter `ess` ships over this same
     seam.
   - *Targets.* Rust, Go, TypeScript, web, the interpreter, `ess verify diff`, the entity runtime,
     generated documentation, synthesis and authoring are untouched: the suite, the IR and the
     report formats keep their bytes.

5. **A second, unrelated adopter.** Two are on record. An adopter in Java "reaches no runner"
   (`story:java-conformance-target`). An adopter with a hand-written HTTP server and a 168-scenario
   suite asked to hold it to the suite over the wire
   (`story:a-command-binds-to-a-protocol-or-deployment-http-route`, lines 359-361).

6. **Cost.**
   - One new versioned wire format for the target protocol, tracked in `FORMAT_RELEASES`
     (`crates/edge/ess-xtask/src/docs.rs:147`) and `website/docs/reference/formats.md`. It is a
     format an adopter implements, so every later change to it is a compatibility decision.
   - New surface on `ess verify conform run`: one `--target` value and the option naming the
     command. The `agentplugins` check that runs every spelled command against the newest release
     is unaffected until the plugin spells the new one.
   - No `ess/N` version, no suite version, no report version, no new `ValidationCode`, no
     migration, no diff classification, no generated-API change.
   - A slower loop than an in-process call, and a child process to start, bound and kill.

7. **What else was considered.**
   - *Change nothing.* The generated TypeScript or Go package with an adapter. It works today and
     is what the requester uses meanwhile; it costs every non-Go, non-TypeScript implementation a
     second toolchain, and one emitter per further language.
   - *Only the live HTTP target* (`story:a-conformance-run-drives-a-live-http-target`). Needs an
     `ess-http/1` binding, covers no command-line program, and a product API has no operation for
     scenario isolation, event observation or entity setup, so most suites would report
     `unsupported`.
   - *A `--target python` emitter.* One more hand-written runner to keep equal to three others.
   - *The requester's alternatives.* "A process to start" and "a documented target protocol over
     stdin/stdout" are taken together. The HTTP transport is not: one transport is one contract to
     version, and a process `ess` starts gives the run a defined start, deadline and end. A service
     that listens on HTTP is reached by an adapter process that calls it.

## Decisions

- **Accept, redesigned.** The need is a gap. Of the requester's three alternative shapes, one is
  built: a target process speaking a versioned protocol on standard input and output, whose
  operations are `ConformanceTarget`'s. The HTTP transport and an "endpoint adapter" flag are not
  built, for the reason in question 7.
- **Needs the operator's word.** The earlier story for this gap,
  `story:a-conformance-suite-runs-against-any-realization`, was archived on 2026-09-15 with its
  epic under approved plan ess-evolution-20260915 revision 1. This story asks for the same
  capability again from a new request, so it moves past `proposed` only on the operator's approval.

## Acceptance

- A design page under `docs/design/` fixes the protocol before code: the format name and version,
  the handshake, one message per operation of `ConformanceTarget` with its members, how an
  unsupported operation, a declared refusal and a target fault are answered, the deadline rule,
  how the process is started and ended, and what `mutate` does with such a target.
- `ess verify conform run --suite <suite> --report-format 2 --report-out <file>` with the new
  target value and a command starts that command and writes a report/2 whose per-scenario outcomes
  equal those of `--target billing` on the same suite, where the command is a test binary that
  serves the built-in billing reference over the protocol. A test compares the two reports'
  `outcomes` and `counts`.
- The exit status follows the existing rules of `run`: with `--strict`, nonzero unless every
  scenario passed; a failed, errored or unsupported scenario is named in the report under its own
  category.
- A target that answers a command with the wrong outcome fails that scenario by name, and no other
  (a test with one mutated answer).
- A target that exits early, writes a line that is not a protocol message, answers a request it
  was not asked, or passes the deadline makes the scenario `error`, never `passed`; `ess` ends the
  process and leaves no child behind. One test per fault.
- An operation the target answers as unsupported makes the scenario `unsupported` with the target's
  reason in the report, and the run fails.
- Two runs of one suite against one deterministic target write reports that differ in no member
  except the completion instant.
- A handshake naming a protocol version this `ess` does not read is refused before the first
  scenario, exit 2, naming the versions it reads.
- The protocol format is recorded in `FORMAT_RELEASES` and `website/docs/reference/formats.md`,
  and `website/docs/guides/verify/runners.md` gains the section for an implementation in any
  language, with a complete adapter for one small specification. `task test-xtask` passes.

## Out of scope

- An HTTP transport for the protocol, and the `ess-http/1` live target
  (`story:a-conformance-run-drives-a-live-http-target`).
- The random-sequence explorer and concurrent histories against an outside target.
- Publishing `ess-conformance` as a crate.
- The `testing-conformance` skill of the agent plugin, which changes in `beyond10x/agentplugins`
  after the release that carries this.

## Scope

- inferred: `crates/verify/ess-conformance/src/` (a new module implementing `ConformanceTarget`
  over a child process; `target.rs` request types gain what serialization they lack)
- inferred: `crates/edge/ess-cli/src/main.rs` (`ReferenceTarget`, the `run` dispatch at
  `:4315-4330`)
- inferred: `docs/design/` (the protocol page), `website/docs/guides/verify/runners.md`,
  `website/docs/reference/formats.md`, `crates/edge/ess-xtask/src/docs.rs` (`FORMAT_RELEASES`),
  `CHANGELOG.md`

## Design inputs

Answers from the requester, 2026-11-21, to be settled on the design page:

- The requester's implementation trees have no standard start command. Each will ship an adapter
  command that takes no arguments, runs from the implementation directory, starts or calls the
  implementation itself, and isolates state per scenario in `begin_scenario`.
- `ess` therefore passes the target nothing but the command as the operator gave it and the
  protocol on standard input and output: no port, no URL, no environment contract.
- Asked for: an option naming the working directory the command starts in, for a run started
  outside the implementation's tree.

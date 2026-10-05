---
title: The Rust runner
description: Hold a Rust implementation to a synthesized suite through the ConformanceTarget trait of the ess-conformance crate, the runner ess itself uses.
---

# The Rust runner

A Rust implementation is held to its suite by the runner `ess` itself uses: the `Runner` of the
`ess-conformance` crate, driving the `ConformanceTarget` trait. There is no generated package.
`ess` writes the suite as one document, and your test runs it.

## What ess writes

The default `--target` is `ir`, the suite document itself. From the `tasks` project of the earlier
pages:

```shell-session ess-tutorial
$ cd ~/tasks
$ ess verify conform synthesize --path . --out suite.json
note: every declared actor may invoke `tasks.list.AddTask`, so no actor is refused it and no `tasks.list.AddTask/grant/denied` scenario is owed
note: every declared actor may invoke `tasks.list.CompleteTask`, so no actor is refused it and no `tasks.list.CompleteTask/grant/denied` scenario is owed
6 scenario(s) (0 authored), 0 refusal(s), written to suite.json
```

`suite.json` is the one generated file: the same scenarios the Go and TypeScript packages embed.
Regenerate it after every change to the specification.

## Depend on the runner

`ess-conformance` is not published to crates.io. Depend on it from the release tag that matches
your pin:

```toml title="Cargo.toml"
[dev-dependencies]
ess-conformance = { git = "https://github.com/beyond10x/ess", tag = "0.53.0" }
```

## What your implementation provides

`ConformanceTarget`, in `ess_conformance::target`. These eight methods have no default and are
yours to write:

```rust
fn identity(&self) -> Result<ImplementationIdentity, TargetError>;
fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError>;
fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError>;
fn execute_command(&self, request: SemanticCommandRequest) -> Result<SemanticCommandResult, TargetError>;
fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError>;
fn observe_events(&self, request: EventObservationRequest) -> Result<Vec<ObservedEvent>, TargetError>;
fn configure_external_outcome(&self, request: ExternalOutcomeControl) -> Result<(), TargetError>;
fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError>;
```

They answer the same questions as the TypeScript runner's `Target`; its
[method table](./typescript.md#what-your-implementation-provides) says what each one answers. A
method the implementation cannot answer returns `TargetError::unsupported(…)`, and the scenario is
reported unsupported rather than failed.

Every other method has a default body that answers unsupported, so a scenario that needs one is
reported unsupported until you implement it: `observe_invocations`, `deliver_event`,
`establish_entity`, `fixture_values`, `execute_command_without_input`,
`configure_external_outcome_repeatedly`, `open_periodic`, `observe_periodic`, `close_periodic`,
`observe_clock_reading`, `mark_instant`, `observe_elapsed` and `scan_view`.

## Run it

Admit the suite, run it, and read the report:

```rust title="tests/conformance.rs"
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, Runner};

/// Your implementation, reached however your tests reach it.
struct Tasks;

impl ConformanceTarget for Tasks {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity {
            name: "tasks".into(),
            version: "dev".into(),
        })
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(request.command.to_string(), "not wired yet"))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(request.view.to_string(), "not wired yet"))
    }

    fn observe_events(
        &self,
        _request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }

    fn configure_external_outcome(
        &self,
        _request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcome", "no test control"))
    }

    fn redeliver_event(&self, _request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "no test control"))
    }
}

#[test]
fn tasks_conforms_to_its_specification() {
    let suite = std::fs::read_to_string("suite.json").expect("read suite.json");
    let admitted = AdmittedSuite::from_json(&suite).expect("an admissible suite");
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &Tasks);
    let report = run.report();
    for failure in report.failures() {
        eprintln!("{failure:?}");
    }
    assert!(report.is_conformant());
}
```

As written, every scenario comes back unsupported and the assertion fails: `execute_command` and
`query_view` are where your implementation is called. `Runner::for_suite` seeds its identifiers
from the suite, so two runs against a deterministic implementation produce the same report.

The built-in targets of `ess verify conform run` (`billing`, `oracle-fixture`, `interpreted`) are
implementations of this trait. [Verify conformance](../../guides/verify/runners.md#a-target-in-rust)
says what a Rust target must preserve.

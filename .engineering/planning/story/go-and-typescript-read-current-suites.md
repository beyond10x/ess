---
format: aep.planning-md/3
id: story:go-and-typescript-read-current-suites
kind: story
status: implemented
title: Go and TypeScript packages read suite formats /28 to /33
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:40:47Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-02T20:40:47Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-02T20:40:47Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The generated Go and TypeScript conformance packages read every suite format the Rust runner reads, from /28 through /33, instead of refusing suites newer than /27.

## Acceptance

- a suite at each of /28 to /33 generates a Go and a TypeScript package that runs it against the reference target with the same verdicts as the Rust runner;
- nested {$instance} values (/32, /33) resolve to the instance in both runtimes.

## Origin

beyond10x/ess#242 (0.43) added /32 and /33; package generation refuses such suites and names the Rust runner.

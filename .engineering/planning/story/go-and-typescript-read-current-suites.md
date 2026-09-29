---
format: aep.planning-md/3
id: story:go-and-typescript-read-current-suites
kind: story
status: draft
title: Go and TypeScript packages read suite formats /28 to /33
relations:
- serves: vision:O2
revision: 1
---
## Outcome

The generated Go and TypeScript conformance packages read every suite format the Rust runner reads, from /28 through /33, instead of refusing suites newer than /27.

## Acceptance

- a suite at each of /28 to /33 generates a Go and a TypeScript package that runs it against the reference target with the same verdicts as the Rust runner;
- nested {$instance} values (/32, /33) resolve to the instance in both runtimes.

## Origin

beyond10x/ess#242 (0.43) added /32 and /33; package generation refuses such suites and names the Rust runner.

---
format: aep.planning-md/3
id: decision-blocker:run-of-a-process
kind: decision-blocker
status: open
title: 'Does a specification state a run of a process: a cause, a command per member of a set, a command once, a continuation after a refused member and an exit status'
summary: 'RN-D1, options A, B, C, D: no run, a run cause of bindings, a run document, or a deployment fact; architecture decision with the owner, no default'
tags:
- architecture
- gap-reports-2026-11-24
relations:
- blocks: story:a-specification-may-state-one-run-of-a-process-with-a-command-once-per-member-of-a-set-and-a-command-once-per
revision: 5
---
Does a specification state a run of a process (a cause, a command per member of a set, a command once, a continuation after a refused member, an exit status), and as a binding cause, as its own document, or not at all: options A, B, C, D of RN-D1, an architecture decision with the owner. There is no default; nothing waiting on it takes one. The question has been with the owner since 2026-11-24; there is no answer yet.

## What a run is

One execution of a process: it has a cause, issues commands (one per member of a set, one per run), continues after a refused member, and ends with a status. ESS states none of it today. A system actor may issue a command, but nothing says how many times one run issues it or over which records; no scenario observes a whole run. Stating it adds a thing a specification declares.

## Options

- A. No run. ESS keeps one command per scenario. A scheduled process is a command-line process bound by `ess-cli`; the fan-out and the counts stay the handler's. Only the exit status per outcome of one forwarded command is added (RN-D2, RN-D3). The second need is declined as out of the language, and the authored multi-await scenario is the named idiom for the rows an author lists. Cost: smallest; no run, no count, "every member" and "once per run" stay unchecked.
- B. A run is a cause of bindings, with a fan-out on the invocation. A binding gains a run cause beside `event:` and `periodic:`; its `invoke:` may repeat over a typed list (the `each:` vocabulary of `affects:`, the selection vocabulary of `binding-list-selection.md`); system-issued commands become admissible as invocations of that cause (lifting the refusal at `ess-domain/src/system_issued.rs:105-125`); a command with no list is issued once; the run's exit status is derived from the outcomes' exits; one suite step observes the issuances of a run. Cost: new binding keys and a source format by policy 2, one suite major, a target capability in Rust, Go and TypeScript, diff classes, generated docs.
- C. A run is its own declaration (a document beside `ess-cli` and `ess-transport`, working name `ess-run/N`): causes (invocation, schedule), ordered steps (`each: <view or selection>`, `once`), a continuation policy, an exit table, compiled against the model, with its own suite step. Cost: a new document kind, IR, diff, docs and every target; the most explicit and the largest.
- D. A run stays a deployment fact. `ess-runtime` already says when a process starts (`schedule:`, `retry_limit`); ESS states nothing of what it issues or how it exits. Both needs are declined with the reason. Cost: none; two figures a reader needs stay outside every suite.

The coordinator recommends B, taken in two steps so that nothing waits on it that need not: the exit status per outcome first (it stands under A and B), then the run cause with fan-out. B reuses bindings and the existing selection vocabulary; C is the safer shape if the owner wants a run to carry its own policy and exit table; A is the fallback if the owner does not want ESS to model processes; D declines both needs. Each of B and C also says where a run's own arguments are declared and how a command's input field names one (RA-D1). The owner decides. The Acceptance of the blocked story is written for B.

## Decisions

- RN-D1: step 1 only, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET (he is away); reversible until a release carries it: the exit status per outcome of one forwarded command in the `ess-cli` binding. The run as a new cause of a binding is a source format: its design page goes to the owner before any build, and this blocker stays open for step 2.

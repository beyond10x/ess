---
title: "0.82 — Binding scenarios that check event identities, refusals that probe the record, and validation that names the file"
description: >
  0.82.0 adds no format: synthesized binding scenarios compare the identity fields of their trigger
  event, the arranged half of a refusal without an identity view probes the record through the
  command, and ess specify validate names the file a refusal was read from.
slug: event-identities-and-file-cited-refusals
tags: [release, ess]
date: 2026-11-01T22:00:00+01:00
release_tag: "0.82.0"
---

0.82.0 adds no source, scenario or suite format, and every change in it is a fix. Two are in the
conformance suites synthesis writes, and one is in what `ess specify validate` cites. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change; this note
covers each of them.

{/* truncate */}

## Binding scenarios compare event identities

- Synthesized binding scenarios compare the identity fields of the trigger's event with the
  instances its invocation was sent, or with the row it acts on. That covers flow, mapping,
  delivery, on-failure, a bounded retry's attempts and final failure, an unbounded retry selected
  per refusal, and a condition's negative witnesses.
- The comparison also holds where the binding arranges a row before it, as command scenarios have
  done since beyond10x/ess#273.
- A target that publishes another identity in the event now fails these scenarios. Before, it
  passed them.
- A model whose binding triggers carry no captured instance into their event keeps its suite bytes.

## The arranged half of a refusal probes the record

- This is the arranged half of an input refusal, on a command that addresses an existing record,
  where no identity view shows the arranged row. After the refused send, the scenario sends a valid
  input for that record. That input belongs to the first branch that addresses the record from the
  state it was arranged in, and the scenario requires that branch to answer.
- A target that moves the record out of every state that branch runs from on refusal, or otherwise
  changes what it answers, and publishes nothing, now fails. Before, the scenario required only the
  error and no event, and such a target passed. A move between two states that branch runs from is
  still not seen.
- Where no such branch has a valid send, the scenario is withdrawn and refused as `ESS-SYNTH-001`,
  naming the branches tried (beyond10x/ess#209).
- A model with an identity view keeps its suite bytes.

## Validation names the file

- `ess specify validate` on a specification of several files names the file a refusal was read
  from. Before, it named `<document>` whenever the refused declaration's name was written in more
  than one file, as a name declared in two files is.
- A refusal made while a file is read is cited in that file. One checked after assembly is cited in
  the file of the copy assembly kept, which is the second when the first does not convert. A name
  written twice in one file names the file and no line.
- The library's `Specification::assemble_traced` and `resolve::diagnose_traced` return and read that
  record; `diagnose_locating` is unchanged.

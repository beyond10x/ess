---
format: aep.planning-md/3
id: decision-blocker:unowned-store-records
kind: decision-blocker
status: open
title: Which construct lets a specification rely on records a system outside its components owns and still check a job over them against a test double
summary: 'SW-D1, options A, B, C: extend provided, a new external records store declaration, or no construct; architecture decision with the owner, no default'
tags:
- architecture
- gap-reports-2026-11-24
relations:
- blocks: story:a-specification-may-declare-the-records-another-system-owns-without-claiming-ownership-and-check-a-job-over
revision: 3
---
Which construct lets a specification rely on records a system outside its components owns and still check a job over them against a test double: options A, B, C of SW-D1, an architecture decision with the owner. There is no default; nothing waiting on it takes one.

## What the check needs

A system-issued job acts on rows in a store another system owns, and that system has no specification. The check wanted: which rows the job removes, in what order, at most how many, and that a dry run changes nothing, against a test double. Today `provided: true` still claims ownership: generated owner artifacts, creator commands needed by synthesis, `ESS-SYNTH-001`. The ADR `architecture-decision-record:a-deployment-may-provide-an-entitys-rows` (accepted) chose a provider that returns `{entity, state}` only.

## Options

- A. Extend `provided`. Rows carry fields to the suite through the existing `setup`; generators stop projecting a provided entity as owned; each provided entity gets a records port (select, count, delete by filter) that a test double implements; no creator command needed. Cost: reopens the accepted ADR's option B and changes generated output of existing provided entities; a port is added to each generated target; no new source key.
- B. A new declaration of an external records store with its entity shapes (own document or section, own protocol enum, effects limited to select, count, delete). Cost: new source key and format row, closed enum, IR field, diff class, per-target ports, docs; the most explicit and the largest.
- C. No construct. Declare the records in an owned stub domain with seed commands and list the rest as unmapped lines. Cost: owner artifacts and exposed creators for rows the system does not own; the suite still checks rows and order.

The coordinator recommends A, the smallest that makes the check real and adds no key. The owner decides. The Acceptance of `story:a-specification-may-declare-the-records-another-system-owns-without-claiming-ownership-and-check-a-job-over` is written for A and changes with the pick.

Related rule: `story:a-command-writes-only-rows-its-own-component-stores` (draft) is the ownership rule that option A and B relax for a defined set of effects.

## Decisions

- SW-D1: stays the owner's.

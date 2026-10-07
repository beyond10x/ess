---
format: aep.planning-md/3
id: story:fast-line-regeneration-fails-on-macos
kind: story
status: draft
title: Regenerating ESS output fails on macOS with a provenance error
tags:
- fast-line
revision: 1
---
## Outcome

Regenerating ESS output on macOS succeeds, or fails with a message that names the cause.

## Origin

Fast-line intake, 2026-10-04. A downstream team reported on 2026-09-29 that regenerating ESS output on macOS fails with "the same provenance error as before", so generated code could not be checked against its specification. The error text was not posted.

## Open question

- Is this the defect fixed by closed beyond10x/ess#306 (generated output cannot be regenerated in another checkout, closed 2026-10-02), or a macOS-specific path or case-sensitivity difference? Needs the exact error text and the ESS version the reporter ran.

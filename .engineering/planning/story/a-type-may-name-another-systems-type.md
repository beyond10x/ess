---
format: aep.planning-md/3
id: story:a-type-may-name-another-systems-type
kind: story
status: archived
title: A type may name another system type
summary: No type can name another system type, so a sibling system document is held as Json.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-26T16:17:17Z", actor: "human:timo", revision: 3}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

No type can name another system's types: a field holding a document of another system's versioned type (the reporter's case is an agent resource document of a sibling system) must be declared `Json`, so its shape is neither validated nor generated. Related: story:cross-system-relation-target (relations across systems, not types).

Feature gap, not a defect.

## Acceptance

- `a_type_names_a_type_of_another_specified_system`: a domain imports a named type from another system's specification by a pinned reference, validation resolves it, and generated Rust, Go and TypeScript use its shape.
- An unresolved or version-mismatched import is a typed refusal naming the reference.

## Duplicate

Archived 2026-10-26 as a duplicate of story:a-specification-imports-another-specifications-types (G-U22), filed the same day before the duplicate was found; the report is recorded there.

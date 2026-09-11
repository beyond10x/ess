---
format: aep.planning-md/1
id: story:verify-compiled-view-delivery
kind: story
status: implemented
title: Verify complete views reach the adopter through compiled IR
tags:
- priority-high
relations:
- decomposes: task:runtime-adopter-gaps-9-13
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/compiled_view_delivery.rs
revision: 5
---
## Outcome

User observed domain view names without complete top-level views. Current inspected EssIr at ir.rs:1389 and ResolvedView at879-931 already serialize complete views; CLI compile at main.rs:2018 uses canonical output. Establish executable stdout and --out witnesses retaining wire name, source, consistency and fields, and resolving domain references. Trace actual adopter compiler/output provenance and provide a candidate adoption path. Do not rewrite a serializer absent a failing reproduction. Test path is inferred; capability locations are cited by read-only scoper.

## Acceptance

Address gap 11 of task:runtime-adopter-gaps-9-13 with executable evidence and actual adoption instructions. Use generic public examples; private consumer names stay in private evidence. No full local gate or unchanged passing test reruns.

## Scope

Confidence: high for cited existing symbols; new test paths are inferred.

- crates/edge/ess-cli/tests/compiled_view_delivery.rs (inferred implementation surface from read-only intake).

## Verified adopter provenance

At private adopter HEAD7f03d513d756ea45df736c7ae14ab95bf0a7cabd, both pinned ESS0.22.2 (declared verified binary digest88100c18992c073983feda6f31eee0f42e6c1f057f6d34375af2bdcccd1c203c) and PR1fd6ba62 compile26 complete top-level view declarations. Both have source,fields,consistency,naming. The adopter conformance reader's ir type exposes only Commands and loads views.json generated from YAML. The measured gap is in this reader, not current ESS serialization. An isolated managed adopter tree now reads the existing IR views directly and removes redundant YAML extraction. Exact compile outputs retained in private runtime-gaps evidence. New public CLI regression checks both stdout and --out, pending execution.

## Completion evidence

The CLI regression compiled and passed 1/1 in 0.01 seconds, asserting stdout and --out byte equality and complete resolved view name, source, consistency, wire naming, and ordered field contracts. Scoped CLI Clippy passed. Actual pinned and PR compilers produced identical 26-view IR with SHA256 934e5d648599c443cee6567591404b83fe77a7d9fcdd7602fb9522001c8015a2. The isolated adopter reads those IR views directly. Independent review confirmed this boundary; the report task defect from that review is tracked and corrected separately. Implemented means the verified source/test and private migration patch are ready, not that the PR is merged or the private patch published.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 583edf298d604674a8a094d12f4fdadfb10123a820e82f732627ee7a2ffe7a14, retained as local-evidence:runtime-gaps/publication-replay/snapshots/583edf298d604674a8a094d12f4fdadfb10123a820e82f732627ee7a2ffe7a14.md. Source creation recorded at 2026-09-11T04:58:03Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->

---
format: aep.planning-md/3
id: review-result:adversary-5w-literal-pass-2
kind: review-result
status: active
title: Adversary pass 2, literal (the-5-waves)
relations:
- reviews: story:literal-fallback-after-else
revision: 1
---
Adversary pass 2 against story:literal-fallback-after-else (#163), aep:adversary, 2026-09-27, the-5-waves wave 1, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 5, red 4 (executed 797→802)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_literal_pass2.rs`. Red: no scenario sends an input a literal fallback reads, so an always-the-literal mutant survives; a `sets:` fallback refusal names the command input, not the entity field; the payload fallback hint suggests `input.rank`, which `else:` refuses. Green: a generated fallback on the same input keeps it sent. Pass-1 correction holds.

Ledger against pass 1: carried 0, new 3, resolved 1 (1 routed to the integration check). Coordinator routing: final correction round; the coordinator verifies it by reading the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 1765, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "without_literal_fallbacks removes the only invocation that sent an input read by a literal fallback, so no scenario asserts the input wins and an implementation that always stores the literal passes."},
 {"file": "crates/specify/ess-domain/src/command/value_expression.rs", "line": 415, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A sets: fallback literal refusal names the command input where a bare sets: literal names the entity field."},
 {"file": "crates/specify/ess-domain/src/command/value_expression.rs", "line": 483, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The misspelled_reference refusal of a payload fallback else: rank says to write input.rank, which else: refuses."}]
```

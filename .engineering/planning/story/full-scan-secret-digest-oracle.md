---
format: aep.planning-md/3
id: story:full-scan-secret-digest-oracle
kind: story
status: implemented
title: The full-cluster scan does not write a guessable digest of a Secret value
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T03:23:26Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T03:23:53Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-09-26T03:24:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
# The full-cluster scan does not write a guessable digest of a Secret value

The Kubernetes full scan replaces a Secret value with its unsalted SHA-256 and exact length (`crates/infra/ess-kubernetes/src/lib.rs:130`). For a low-entropy value that file confirms guesses. Use a keyed or salted digest, or record presence only. Pre-existing; found by the M8 adversary review.

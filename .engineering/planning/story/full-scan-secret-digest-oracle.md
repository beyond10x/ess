---
format: aep.planning-md/2
id: story:full-scan-secret-digest-oracle
kind: story
status: draft
title: The full-cluster scan does not write a guessable digest of a Secret value
relations:
- serves: vision:O2
revision: 1
---
# The full-cluster scan does not write a guessable digest of a Secret value

The Kubernetes full scan replaces a Secret value with its unsalted SHA-256 and exact length (`crates/infra/ess-kubernetes/src/lib.rs:130`). For a low-entropy value that file confirms guesses. Use a keyed or salted digest, or record presence only. Pre-existing; found by the M8 adversary review.

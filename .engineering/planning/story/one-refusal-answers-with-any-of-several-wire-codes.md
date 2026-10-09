---
format: aep.planning-md/3
id: story:one-refusal-answers-with-any-of-several-wire-codes
kind: story
status: draft
title: One refusal is answerable with any of several wire codes
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#508
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

One refusal can be declared answerable with any of several wire codes, and conformance accepts
any listed code for it.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/508 (ess 0.56.0): RFC 6749 §4.1.3 with §5.2 lets a token request
missing its `redirect_uri` be answered `invalid_request` or `invalid_grant`. An error's wire form
names one code; declaring one fails a conformant implementation that answers the other.

## Acceptance

- An error's wire naming may list several codes; an empty list and a duplicate are refused.
- A refusal scenario passes on any listed code and fails on an unlisted one.
- `ess verify diff` rates removing a listed code as breaking for readers and adding one as
  breaking for callers that switch on the code.

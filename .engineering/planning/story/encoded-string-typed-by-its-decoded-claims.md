---
format: aep.planning-md/3
id: story:encoded-string-typed-by-its-decoded-claims
kind: story
status: draft
title: An encoded string is typed by the structure of its decoded claims
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#507
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A string whose wire value is an encoding of a structured value (a JWS compact serialization) can
be typed with the structure of its decoded claims, and validation and conformance reach the claim
constraints.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/507 (ess 0.56.0): an OpenID Connect ID token can only be typed `String`;
its claim constraints (`iss`, `sub` at most 255 ASCII characters, `aud` contains the `client_id`,
`exp`, `iat`, `nonce` equal to the request's) cannot be stated, and 24 requirements of the
requester's model stay unmapped.

## Acceptance

- A type kind declares an encoded value with a format and a claims struct; the struct's field
  guards constrain the decoded claims. Signature verification stays out of the type.
- Conformance decodes a returned value and checks the claims; a target returning a token whose
  `aud` lacks the client fails.
- A design page under `docs/design/` precedes code; it decides the closed list of formats.

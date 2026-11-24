---
format: aep.planning-md/3
id: specification:a-command-only-some-builds-serve
kind: specification
status: draft
title: A command only some builds serve
summary: Decline of beyond10x/ess#816 with the idiom, run on 0.98.0
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- specifies: story:the-components-page-says-a-command-only-a-development-build-serves-stays-in-the-component-that-owns-its-domain
revision: 1
---
## Reproduction

Reproduced on 0.98.0, both models under `run/` (generic names, `format: ess/36`):

- `run/devonly/system.yaml`: domain `devonly.access`, entity `Session` (`Active` to `Ended`),
  commands `LocalSignIn(user)` (creates a session; a comment says development builds only) and
  `SignOut(session_id)` (`unknown-session`, `signed-out`, `already-ended`); one component
  `access-service` owns the domain and accepts both.

  ```console
  $ ess specify validate --path devonly
  devonly v1 — 1 file(s), valid
  $ ess verify conform synthesize --path devonly --out devonly-suite.json
  5 scenario(s) (0 authored), 0 refusal(s), written to devonly-suite.json
  $ ess verify conform run --path devonly --suite devonly-suite.json --target interpreted --report-format 2
    5 scenarios: 5 passed, 0 failed, 0 error, 0 unsupported
  ```

  `SignOut`'s scenarios arrange their session through `LocalSignIn`.
- `run/devonly-split/system.yaml`: the same, with `LocalSignIn` accepted by a second component
  `dev-sign-in` that owns no domain:

  ```console
  $ ess specify validate --path devonly-split
    - error[ESS-COMPONENT-004]: `dev-sign-in` accepts `devonly.access.LocalSignIn`, and `access-service` owns `devonly.access`
  ```


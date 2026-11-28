---
format: aep.planning-md/3
id: story:a-secret-value-appears-in-no-output-of-the-process
kind: story
status: draft
title: A conformance suite fails when an event, an answer or an output line of the process carries a value the specification declares secret, a field's or one read from its environment
summary: A declared secret value, a marker per run and a scan of every observed channel; security-marked, SV-D1 to SV-D6 with the owner, no default
tags:
- external-request
- gap-reports-2026-11-24
- security
relations:
- serves: vision:O2
- depends_on: story:field-sensitivity-class
- informed_by: decision-blocker:run-of-a-process
revision: 1
---
## Summary

A specification can say that a value is secret, and a conformance run fails when an event, an
answer or another output line of the process under test carries it. The value is a field's, or one
the process reads from its environment through a runtime secret slot. Today ESS has two halves and
no join: the draft `story:field-sensitivity-class` classifies a field and checks nothing, and the
runtime document names a secret slot by identity and never says which output must stay free of it.

Security-marked: every open choice is the owner's (`decision-blocker:secret-values-and-their-output-lines`,
SV-D1 to SV-D6). No choice has a default. Nothing in this story is built before the owner answers.

## Acceptance

Each line holds under the owner's answers; the spelling of the declarations is this story's design
once SV-D3 and SV-D6 are answered.

- A specification declares a value secret in the place SV-D3 names, and a declaration that no
  channel or marker could ever reach is refused at validation with a diagnostic code and a repair.
- A synthesized suite gives each declared secret value a marker that is unique per value and per
  run, sends it where the declaration says it enters the process (an input field, or the declared
  environment variable), and after every operation scans each channel SV-D2 names for it, by the
  comparison SV-D1 names.
- A target that writes the marker into any scanned channel fails the scenario with a check that
  names the channel, the declared secret and the operation, and never quotes the value or a
  fragment of it (as `a_fault_never_quotes_the_answer` holds for protocol faults,
  `docs/design/conformance-target-process.md` § Protocol faults).
- A target that writes no marker passes. A channel the run cannot observe is reported as not
  checked and refused as the owner's SV-D2 answer says; it is never reported as passed.
- Each guard of the check is held by a mutation: a target that echoes the marker in an event field,
  in an answer, in an error and (when SV-D2 includes it) in a standard-error line fails, and the
  same target without the echo passes.
- The compiled IR and the suite carry the declaration; a new format version records it, older
  versions refuse it by name, and specifications without it keep their compiled bytes.

## Fit review

1. **The need.** A value a process reads is declared secret; no output line of the process (an
   event, a log, an error line) may hold it. Example, generic: a scheduled process reads two
   passwords from environment variables, issues commands, and writes one report event per record.
   A bug writes `connect failed for <value>` into the event's text field. The specification can
   state neither that the value is secret nor that the event must not carry it, and the suite sends
   no value to look for. The requester's proposed keys (`secret: true` on an event field,
   `sensitive: true` on a newtype, a `protected` source with `env:`) are theirs and are evidence of
   the need, not the design.
2. **Class.** Gap: the fact is not expressible and not checkable (the requester's probes on 0.97.4
   refused each spelling as an unknown field; `main` at 0.99.0 refuses them the same:
   `CHANGELOG.md` 0.99.0 names none).
3. **Expressible today?** Partly, and not checked. `ess-runtime` names a secret slot by identity
   and the environment variable it is read into (`crates/generate/ess-deployment/src/runtime.rs:51-66`),
   projected as a `secretKeyRef` (`environment.rs:1197`); it says nothing about outputs.
   `one_time_response` forbids one issued value in every position but its marked response field
   (`docs/design/one-time-response-values.md:26-32`), for a command's own response, not for a
   value the process is handed. An adopter's own test that greps its logs is an idiom, and it moves
   the figure the reader asks about out of the specification (policy 1 of
   `docs/design/generic-language-features.md` fails).
4. **Fit.** The check reuses the precedent of the one-time check: a marker, an exact
   case-sensitive substring scan of every observed string after each operation, no decoding, and a
   redacted fixed diagnostic (`one-time-response-values.md:26-32`; `CHANGELOG.md:1119-1124`,
   "the scan of every catalog event after each operation"). Three things do not fit what is there
   and are decisions: the target process inherits `ess`'s environment and `ess` adds no variable
   (`conformance-target-process.md:61-62`); the target's standard error is passed through and never
   parsed (same lines); and the `protected` source offers `file`, `stdin` and `hidden_tty`, with
   "no argv value channel" and no environment source (`cli-presentation-binding.md:180-183`).
5. **A second adopter.** A service that reads a signing key from its environment and writes request
   logs; a CLI that takes a token from a file and prints a diagnostic on failure. Neither may echo
   the value.
6. **Cost.** A new declaration (SV-D3, SV-D6) and so a format version for the specification, a
   suite format version for the marker and the scanned channels, new diagnostics, a change of the
   conformance target's contract if the runner sets a variable or reads standard error (SV-D2,
   SV-D4), and per target a decision whether it can observe the channel.
7. **Considered.** Change nothing (the adopter greps; rejected: unchecked figure). The requester's
   per-field `secret: true` only (classifies, joins nothing, covers no environment value). A static
   rule alone (refuse a secret-class field in any output position; fail-closed but blind to a
   value an unrelated code path formats into a text field). The chosen shape is a declaration plus
   a dynamic scan, with the static half as the owner's SV-D5.

Findings covered: a field declared secret (the declaration half rides on
`story:field-sensitivity-class`); a value read from the environment declared secret; a check that
no event, log or other output line carries it. Borders: `story:field-sensitivity-class` (declares
the class; depends on it), `decision-blocker:run-of-a-process` (a scheduled process and its report
events), `story:full-scan-secret-digest-oracle` (a digest of a secret is a derived form, SV-D1), the
Kubernetes credential edge of `AGENTS.md` (its redaction tests and mutation guard are the model for
the guard test here), the stories on disclosure cells in synthesis (what an answer may reveal;
`story:disclosure-scenarios-bind-the-caller`).

## Decisions

- Draft. Defer, with a blocker: `decision-blocker:secret-values-and-their-output-lines`
  (SV-D1 to SV-D6, no default; each names its fail-closed option). The `ess-cli` `protected`
  `env` source is SV-D6's second line and is not built by this story.
- Not decided here: whether the declaration lives in the runtime document or the model (SV-D3),
  and whether a standard-error line is observed (SV-D2).

## Scope

- `crates/specify/ess-domain` (the declaration, once SV-D3 is answered), `crates/verify/ess-conformance`
  (marker, channel scan, redacted diagnostic, mutation tests: `synthesize.rs`, `runner.rs`,
  `src/process/`), `crates/generate/ess-deployment/src/runtime.rs` (only if SV-D3 is A).
- `docs/design/conformance-target-process.md` (only if SV-D2 or SV-D4 changes lines 61-62),
  `docs/design/cli-presentation-binding.md` (only if SV-D6 adds `env`), `CHANGELOG.md`.

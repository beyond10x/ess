---
format: aep.planning-md/3
id: decision-blocker:secret-values-and-their-output-lines
kind: decision-blocker
status: open
title: 'Which value is secret, where that is declared and what a conformance run counts as a line that carries it: SV-D1 to SV-D6'
summary: 'Six security decisions with the owner, no default, each with a fail-closed option: match rule, channels, declaration site, marker delivery, static refusal, class meaning and the protected env source'
tags:
- architecture
- gap-reports-2026-11-24
- security
relations:
- blocks: story:a-secret-value-appears-in-no-output-of-the-process
revision: 1
---
Which value is secret, where that is declared, and what a conformance run counts as a line that carries it: SV-D1 to SV-D6. A decision that touches a security boundary has no default; the owner decides. Each decision names its fail-closed option, the one where a line that could carry the value is refused or reported unchecked, never passed.

## Why this is open

`story:a-secret-value-appears-in-no-output-of-the-process` needs a declaration of a secret value (a field's, or one a process reads from its environment) and a check that no output carries it. The evidence is in that story's fit review. Four existing lines are touched: the conformance target inherits the environment and its standard error is never parsed (`docs/design/conformance-target-process.md:61-62`), the one-time check is an exact substring scan with no decoding (`docs/design/one-time-response-values.md:26-32`), `ess-cli` `protected` has `file`, `stdin` and `hidden_tty` and no environment source (`docs/design/cli-presentation-binding.md:180-183`), and a secret slot of `ess-runtime` is declared by identity (`crates/generate/ess-deployment/src/runtime.rs:51-66`).

## SV-D1: what counts as a line that carries the value

- A. Exact, case-sensitive UTF-8 substring of the marker, no decoding: the one-time check's rule. Cost: none new; a value a program re-encodes (base64, URL-escaped, hex), truncates or hashes passes.
- B. A, and the marker's declared encodings: JSON string escapes, URL percent-encoding, base64 (standard and URL alphabet), lower and upper hex. Cost: a fixed list in the contract; still no cryptographic inference; a false pass for a custom encoding.
- C. B, and a prefix or suffix of at least a stated length, and an unsalted digest of the marker (`story:full-scan-secret-digest-oracle` writes one). Cost: false failures for a short common fragment; a length constant to choose.

Fail-closed: C. No default. What each does not claim: none of them proves absence of every derived form.

## SV-D2: which channels are scanned

- A. Declared events, command answers, view answers and errors: what the runner observes today.
- B. A, and the target's standard-output lines outside the protocol and its standard-error lines. Cost: changes `conformance-target-process.md:62` ("standard error is `ess`'s standard error, passed through and never parsed"): `ess` reads and keeps lines, a bound on them, and the scan never prints one.
- C. B, and files or sinks a declaration names (a log file path). Cost: a new declaration and a path the runner reads.

A channel a run cannot observe is reported as not checked (and fails the check) or as passed: fail-closed is the first. Fail-closed: C, with an unobservable channel failing. No default.

## SV-D3: where a secret value is declared

- A. In the runtime document: the secret slot says which model class or field its value is (a key on `SecretSlot`). Cost: a new `ess-runtime` format version (`/3`); the join between the deployment document and the model is new, and the model alone cannot be checked without the runtime document.
- B. In the model: a process input of the declared secret class, read from a named environment variable. Cost: needs a model of a process and its inputs, which `decision-blocker:run-of-a-process` (RN-D1) has not settled; a source namespace beside `event.*`.
- C. Only the field's class (`story:field-sensitivity-class`); the environment value is not declared, and the adopter supplies a marker through the target's own configuration. Cost: the environment half stays unchecked by the specification (policy 1 fails for it).

Fail-closed: A or B: an undeclared environment secret cannot be checked and is the loophole C leaves. No default.

## SV-D4: how the marker reaches the process

- A. The runner sets the declared environment variable in the target child's environment to the marker. Cost: changes `conformance-target-process.md:61` ("`ess` adds no argument and no environment variable"), a target contract version.
- B. The target reports the secrets it was given, by identity and value, over the protocol. Cost: the value travels on a protocol line and the runner must keep it out of every fault text.
- C. Only values sent as inputs carry markers; the runner sets nothing in the environment. Cost: an environment secret is never marked.

Fail-closed: A. No default.

## SV-D5: a secret-class value in an output position, statically

- A. Refused at validation: a field of a secret class may not appear in an event, view, answer or error, except the one marked response field of a one-time response. Cost: a diagnostic code; a legitimate masked value needs a different type; blind to a value formatted into an unrelated text field (hence the dynamic check stays).
- B. Allowed; only the dynamic check of this story looks. Cost: a specification can declare an event that carries the secret, and only a run finds it.
- C. A holds for events and errors; views and answers are allowed where the class vocabulary declares it so (SV-D6). Cost: a per-class flag.

Fail-closed: A. No default.

## SV-D6: what makes a class secret, and the `ess-cli` `protected` `env` source

- A. The class vocabulary of `story:field-sensitivity-class` declares each class `output: forbidden` or `output: allowed`; the check and SV-D5 read it. Cost: a vocabulary key; a personal-data class such as a phone number is `allowed` and is out of this check.
- B. One reserved class name (`credential`) is secret and every other class is not. Cost: a hard-coded name in a vocabulary the specification declares.
- The `ess-cli` `protected` source and an environment variable. Three options, independent of A and B:
  - E1. No environment source: `protected` keeps `file`, `stdin` and `hidden_tty`; a binding naming `env:` is refused with a diagnostic that names the three. The design page gives no reason for the absence; I do not know why it was left out.
  - E2. A fourth, mutually exclusive source `env: <VARIABLE>` on a `String` field, no argv value channel, the variable name argv metadata, the value never in output or Debug (`cli-presentation-binding.md:344`). Cost: an `ess-cli` format pair (`ess-cli/3`, `ess-cli-plan/3`), a runtime reading path with none of the file-mode admission checks, and the variable inherited by every child the handler starts.
  - E3. The `protected` source names a secret slot of the runtime document, so the variable name is declared once in `ess-runtime`. Cost: a join between two documents, as SV-D3 A.
  
Fail-closed: A, and E1. No default.

## Not decided here

The spelling of any key, the format version numbers, and whether the check also covers the interpreted target's own outputs.

# B: an authored scenario cannot witness a creating command only a system actor may issue

ess 0.99.8 (`out/version.txt`). Model: `desk.yaml`, one entity (`Item`), one command
`RecordItem` granted only to `Desk` (`system: true`), whose outcome `recorded` creates the row.
Shaped like a real purchase-recording command. Scenarios use `type: ess-scenario/6` in the same layout
as a working authored scenario (`timeline:` act with `outcome:` and `events:`, then `assert:` on a
view).

- B1 `scenarios/b1-sent-as-system-actor.yaml`: the act names `actor: repro.desk.Desk`.
- B2 `scenarios/b2-awaited.yaml`: the same act with no `actor:` (awaited).

## Commands

```console
ess specify validate --path .
ess verify conform author --path . --scenarios scenarios/b1-sent-as-system-actor.yaml --out out/b1-sent-as-system-actor.json
ess verify conform author --path . --scenarios scenarios/b2-awaited.yaml --out out/b2-awaited.json
ess verify conform synthesize --path . --out out/suite.json
```

## Observed

`validate`: `repro v1 — 2 file(s), valid`.

B1 (`out/b1-sent-as-system-actor.txt`, exit 1):

```text
refusal[ESS-AUTHOR-044]: `repro.desk/authored/b1-sent-as-system-actor` in scenarios/b1-sent-as-system-actor.yaml
  `repro.desk.RecordItem` is sent as `repro.desk.Desk`, which is the system itself; the system issues it, so the act names no actor and the scenario awaits it
  help: no caller can be the system: name no `actor:`, and the act awaits the system's step; to check that an outside caller cannot issue it, name an actor the specification does not grant it, with `refused: not_granted`
0 authored scenario(s) from 1 file(s), 1 refusal(s), suite ess-conformance/34, written to out/b1-sent-as-system-actor.json
```

B2 (`out/b2-awaited.txt`, exit 1):

```text
refusal[ESS-AUTHOR-045]: `repro.desk/authored/b2-awaited` in scenarios/b2-awaited.yaml
  `repro.desk.RecordItem` is issued by the system and awaited on a record, and its outcome `recorded` creates the record it addresses, so no record exists before the step
  help: state the input the outcomes name as `instance:` under `input:`, a literal or `{$instance: name}`; a step the system takes to create a record, or over rows with no subject, is not awaited in this release
0 authored scenario(s) from 1 file(s), 1 refusal(s), suite ess-conformance/34, written to out/b2-awaited.json
```

Synthesis (`out/synthesize.txt`, exit 0) refuses the same outcome:

```text
refused: refusal[ESS-SYNTH-023]: outcome repro.desk.RecordItem/recorded has no scenario `repro.desk.RecordItem/outcome/recorded`
  `repro.desk.RecordItem` is issued by the system itself, and its outcome `recorded` creates the record it addresses, so no record exists before the step; a scenario awaits a system-issued step on an existing record and never drives one
  help: a step the system takes to create a record, or over rows with no subject, is not awaited in this release; grant the command to a caller as well to keep it driven, or move the step onto an existing record that its outcomes name with `instance:`
0 scenario(s) (0 authored), 1 refusal(s), written to out/suite.json
```

So no route in 0.99.8 gives `RecordItem/recorded` a scenario without granting the command to a
non-system caller.

## Expected

B2 compiles: an awaited system step whose outcome creates its record is awaited on the creation
it claims (the event and the view row the act states), so the outcome can be witnessed. B1 stays
refused (`ESS-AUTHOR-044` is the intended answer; its help points at B2).

---
format: aep.planning-md/3
id: story:a-foreign-scenario-type-is-refused-by-format-before-its-keys
kind: story
status: draft
title: A scenario document of a foreign type is refused for its type, not for its first unknown key
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Source: connectors (beyond10x/connectors), reported on ess 0.67.0, forwarded gap row 20. The repository defines its own trace format, `type: mcp-outbound-lifecycle/1` (given/when/then prose with `story`, `document`, `state`, `outcome`, `wire_code`, `given`, `when`, `then`, `never`), for its 23 outbound MCP scenarios. Handed to the authored-scenario reader, every one of them is refused for its first non-ESS key, `story`, and not for its foreign `type:`. The adopter read that refusal as "ESS refuses an unknown key rather than ignoring it" and left the directory out of its shared-IR gate (connectors `crates/connectors-build/src/gate.rs:327-334`).

Fit: ESS does not and should not accept a repository-defined trace format into the shared IR. A foreign document explicitly listed as a scenario is an intended input to the scenario reader and is refused visibly, never discarded or reinterpreted (`docs/design/review-authored-discovery.md:85`, case A12 at `:206`). The scenario formats are closed: `ess-scenario/1` to `/5` (`website/docs/reference/formats.md`). Prose given/when/then has nothing a compiler can resolve against the model. So the acceptance the adopter asked for is not adopted.

The gap is the diagnostic order. The reader parses the whole document under `deny_unknown_fields` before it looks at `type:`, so a document that is not an ESS scenario at all is reported as an ESS scenario with a misspelt key. The format refusal (ESS-AUTHOR-002, which names the found format and the accepted ones) exists but is reached only when the foreign document happens to use no key outside the ESS field list.

## Reproduction

ess 0.68.0, `ess verify conform author --path examples/billing --scenarios <dir>` over one adopter file (`cancellation-closes-the-response-stream.yaml`, `type: mcp-outbound-lifecycle/1`), exit 1:

```text
refusal[ESS-AUTHOR-001]: mcp-outbound-lifecycle/scenarios/cancellation-closes-the-response-stream.yaml
  unreadable: unknown field `story`, expected one of `type`, `domain`, `scenario`, `summary`, `fixtures`, `arrange`, `timeline`, `assert` at line 2 column 1
  help: the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a key it does not know is refused rather than ignored
```

Control, same binary: a file with only `type: mcp-outbound-lifecycle/1`, `domain`, `scenario` and `summary` gets the right refusal:

```text
refusal[ESS-AUTHOR-002]: only-type.yaml
  `mcp-outbound-lifecycle/1` is not `ess-scenario/1`
```

## Where it lives

- `crates/verify/ess-conformance/src/authored.rs:370-394`: `Document`, `#[serde(deny_unknown_fields)]`, the field list in the refusal.
- `crates/verify/ess-conformance/src/authored.rs:2524-2533` (`compile_one`): full `ess_yaml::from_str` into `Document`, then `document_format_refusal` (`:2614`). The format is checked only after every key has been admitted.
- `crates/verify/ess-conformance/src/authored.rs:2496-2501` (`arrangements`): the same full parse.
- Same shape as `story:compiled-transport-ir-is-refused-by-name` on another reader.

## Acceptance (provisional)

- `authored_foreign_type_is_refused_by_format_before_unknown_keys`: a document whose `type:` is not an `ess-scenario/N` is refused with ESS-AUTHOR-002 naming the found format, whatever other keys it carries; no ESS-AUTHOR-001 for its keys.
- `authored_ess_scenario_with_unknown_key_still_refused`: an `ess-scenario/N` document with a misspelt key keeps the ESS-AUTHOR-001 unknown-key refusal.
- `authored_missing_type_is_refused_by_name`: a document with no `type:` says the key is missing rather than listing an unknown key.
- The format refusal's help says a repository-defined trace format is not a scenario input and is left out of `--scenarios` and `ess-inputs.yaml`, rather than suggesting a rename of its `type:`.

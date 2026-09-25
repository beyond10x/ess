---
format: aep.planning-md/2
id: story:specify-upgrade-command
kind: story
status: draft
title: ess specify upgrade moves a specification to the next source format and checks the delta
relations:
- serves: vision:O2
revision: 1
---
# ess specify upgrade moves a specification to the next source format and checks the delta

`ess specify upgrade --to ess/N+1` rewrites the `format:` line and every mechanical rewrite the new reader demands, then runs `ess verify diff` old → new and refuses unless the delta is empty or only in the construct families the release's change fragment declares. Today a document moves majors by hand-editing `format:`; there is no upgrade command (the only `upgrade` is the recovery tool's `upgrade --install`, `ess-cli/src/recovery/process.rs:129`). Filed by ESS-EVOLUTION.md revision 3 in place of consumer accounting.

# Retained command result replay: verified source release

ESS 0.29.0 is published at the annotated tag object a6246e992bce4314a958a7df2c03a568012cf101,
resolving to 8bef63a21766c54f0d809b4decf1d9f7bd587118. This commit is on main.
The source integrated through [PR 59](https://github.com/beyond10x/ess/pull/59).
The bounded operator approval is recorded on
decision-blocker:retained-result-release-qualification and integrated through
[PR 60](https://github.com/beyond10x/ess/pull/60).

## Executed qualification

At the exact tag commit, task check SKIP_CONSUMER_CHECKS=true exited zero:
342 summaries; 3331 passed, 0 failed, 14 ignored. The ignored cases are reported rather than
counted as passes. task site-lab exited zero. Original logs, per-command
statuses and the exact head remain in the private release evidence directory.
Independent implementation and adversary records remain in this planning store.

[Release run 35710714389](https://github.com/beyond10x/ess/actions/runs/35710714389)
completed successfully: the exact-tag gate, browser laboratory, four native
packages and publication passed. The output-ownership job is excluded on this
release event by the existing workflow; both native ownership jobs passed on
the reviewed source PR. No workflow assertion or default local gate changed.

The [published release](https://github.com/beyond10x/ess/releases/tag/0.29.0)
is neither draft nor prerelease. Its four platform archives and SHA256SUMS
were downloaded independently after publication. Every archive matched
SHA256SUMS and the release API digest, and contained the binary, LICENSE and
README. The downloaded Linux binary reports ess 0.29.0. Its SHA256 is:
88a1e82774dd9e5a6913c7700416e3621bb82b57ace624531e8d21248b409c54

task release-status then exited zero and reported all pushed version tags
on origin/main with published releases, dated changelog sections and recorded
changes. The API lists github-actions[bot] as release author, from the
repository's reviewed release workflow.

## Preserved qualification boundary

The operator approved the existing CI/release profile for this release.
Default local task check still refuses at the separately retained consumer
accounting admission; that broader accounting story remains open. No consumer
classification, coverage assertion, baseline or default task setting was
relaxed. The separate exact-tag result above records its profile explicitly.

This closes the ESS source-and-release dependency. It does not establish EKR
durable receipts, kernel restart behavior, migration or runtime conformance.
The final source7 capability includes retained result retries and complete
wrong-state refusal observations; the versioned suites and reports are tested
in ESS. EKR must adopt the released compiler and execute its own target.
Documentation publication is asynchronous and is not claimed here.

## Coordinator measurement correction

The first release-summary record counted the numeric columns incorrectly and
reported zero passes/ignores. Inspection of the original runner lines corrected
the total above before the closure PR was opened. The measurement now adds
fields 4, 6 and 8 of lines beginning with "test result: ok.", retaining the
original logs and append-only journal. No test result or qualification changed.
Owner: coordinator.

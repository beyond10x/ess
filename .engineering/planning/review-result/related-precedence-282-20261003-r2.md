---
format: aep.planning-md/3
id: review-result:related-precedence-282-20261003-r2
kind: review-result
status: active
title: Final whole review of source22 related-refusal precedence
relations:
- reviews: story:feature-request-282
revision: 1
---
approve

# #282 related-refusal / wrong-state precedence — final independent review 2 of 2

Candidate `31e7362f89464846f069591b313529bbc2f65f9d` passes the final whole review. The review covered the complete source-22 admission, interpreter precedence, synthesis arrangements, generated obligation text, tests and both pass-1 corrections.

```findings
[]
```

The corrected preflight classifies each selected alternative under its own subject authority. It replaces only absent or invalid moving subjects with lifecycle refusals, retains valid nonmoving and external alternatives, and takes valid alternatives through the ordinary execution/response path only when at least one lifecycle refusal answers. If every alternative may act, it prepares no response and proceeds to the related-refusal phase. This preserves `Open`, `Forced` and `Withheld` external semantics, deduplication, Unknown/defer behavior, error propagation and one-time response authority.

Admission remains limited to `ess/22` commands combining `wrong_state` with present related predicate refusals. Older sources remain refused; related acceptances remain outside the widening; commands without `wrong_state` retain prior behavior; overlapping related refusals remain ambiguous. Early missing-related, input-refusal and addressed-row-existence behavior is unchanged. No Optional/stored-via or #304 behavior was admitted.

The pass-1 prospective Open-external regression is committed in the candidate. Independent execution of the exact retained binary (SHA-256 `dafd8e12bc67e5d6367dec5d602aec1b4cff235ad293f1b709e146a8d899d71a`) passed all 14 tests, including Open external retention, allowed-state related refusal, Forced/Withheld controls, missing-related, nonmoving, acceptance-order and wrong-state cases. The binary's source fixture blob matched the clean frozen candidate and author checkout. This was independent execution of a supplied binary, not fresh compilation.

Author evidence records 14/14 corrected interpreter tests, strict all-target Clippy and formatting green. The earlier broader run remains red at 31 targets / 166 failing tests: 18 failures were reproduced before #282, while the rest remain unclassified. This approval does not relabel those failures or claim the queued neighboring/integration checks are green, and it does not close #312 compatibility work.

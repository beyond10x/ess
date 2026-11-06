---
format: aep.planning-md/3
id: review-result:ess-0890-u1-ride-alongs-adversary
kind: review-result
status: active
title: 'ESS 0.89.0 U1 ride-alongs adversary: three introduced findings fixed in the wave'
relations:
- reviews: approval-record:ess-0890-wave
prose_only: findings fixed in the wave are stated in prose with their fix commits
revision: 1
---
unit: ESS 0.89.0 U1, eight ride-along pull requests (741, 742, 743, 744, 745, 746, 747, 749) merged into integration/wave-2026-11-06-ess, adversary tests 41585322565
verdict: findings fixed in the wave
origin: introduced 3 / pre-existing 0
fixes: 6de33743086, a563bba58e4

Findings:

1. 745: the browser toggle read its row state differently from the terminal; it now reads the state through the same contains test.
2. 745: the contains test split its operand differently from the terminal's whitespace split; both now split alike.
3. 746: the tags input ignored the declared suggests filter and the checker did not visit it; the input applies the filter and the checker visits it.

The re-pin commit 0af69bc335b names no story: the ride-alongs carry none, so this record is their review.
A pre-existing gap found on the way (the terminal draws a bound form field from the draft rather than its binding) is filed as a draft and not taken by this wave.

Evidence: PR https://github.com/beyond10x/ess/pull/762 CI run 42274504324 green at 0307e8bc99a; squash commit ec2cb77cad2 on main.

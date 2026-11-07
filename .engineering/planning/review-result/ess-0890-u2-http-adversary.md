---
format: aep.planning-md/3
id: review-result:ess-0890-u2-http-adversary
kind: review-result
status: active
title: 'ESS 0.89.0 U2 bare body and literal-first routes adversary: findings fixed, one pre-existing gap drafted'
relations:
- reviews: story:a-command-route-takes-one-input-as-its-bare-request-body
- reviews: story:a-literal-path-segment-takes-precedence-over-a-variable-one
prose_only: findings fixed in the wave are stated in prose with their fix commits
revision: 1
---
unit: ESS 0.89.0 U2, bare request body and literal-first nested routes (commits b195e93948e, 87166ae5b03), adversary tests b346f4b1066
verdict: findings fixed in the wave
origin: introduced 5 / pre-existing 1
fixes: d3b46ff554e..9daed8906c7, merged 205b3011670

Findings:

1. A percent-encoded literal segment was compared undecoded; literals are now compared after decoding unreserved escapes.
2. The Go server did not check the members of a bare request body; it now does.
3. A bare request body on an input under a refinement was mishandled; fixed with a test.
4. Literal precedence applied to crossed overlaps; it now applies to nested overlaps only, and crossed overlaps stay ESS-HTTP-011.
5. Documentation rows for the new diagnostics were missing; added.
6. Pre-existing: the Go server refuses an undeclared member only at the top level. Filed as draft story:the-go-server-refuses-an-undeclared-member-at-any-depth.

Evidence: PR https://github.com/beyond10x/ess/pull/762 CI run 42274504324 green at 0307e8bc99a; squash commit ec2cb77cad2 on main.

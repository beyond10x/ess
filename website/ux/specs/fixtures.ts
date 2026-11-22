// Canned inputs shared by the specs; no test here.

// A finding record with every header field, for the known-failure rule's tests. `pattern` is the
// record's failure_pattern: the message its asset fails with, `none` when it names no failure.
export const findingRecord = (id: string, status: string, asset: string, pattern = 'none') =>
  `---\nid: ${id}\ntitle: t\nclass: layout\nseverity: S2\nmetric: m\nstatus: ${status}\nfound_by: L2\nevidence_recording: none\nevidence_screenshot: none\nevidence_lines: l\nfailure_pattern: ${pattern}\nregression_asset: ${asset}\n---\n\n# ${id}\n`;

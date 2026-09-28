// A row view for an authored `satisfies` predicate over the suite/14 string operators
// (beyond10x/ess#95). Modes: `correct`, `uk-caller` (a row starts with `+44`), `personal-mail`
// (a row's mail does not end with `@corp.example`), `routine` (a subject lacks `urgent`),
// `numeric-subject` (a subject that is not text).
const mode = process.env.ESS_TARGET_MODE ?? 'correct';

class Rows {
  identity() {
    return { name: 'text-match-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command }) {
    throw new Error(`unexpected command ${command}`);
  }
  queryView({ view }) {
    if (view !== 'routing.calls.Rows') {
      throw new Error(`unexpected view ${view}`);
    }
    const rows = [
      { caller: '+4930123', email: 'ops@corp.example', subject: 'urgent: line down' },
      { caller: '+1555', email: 'desk@corp.example', subject: 'not urgent' },
    ];
    if (mode === 'uk-caller') rows[1].caller = '+44207';
    if (mode === 'personal-mail') rows[0].email = 'ops@home.example';
    if (mode === 'routine') rows[1].subject = 'routine';
    if (mode === 'numeric-subject') rows[0].subject = 7;
    return { rows };
  }
  observeEvents() {
    return [];
  }
  configureExternalOutcome() {
    throw new Error('nothing here is externally decided');
  }
  redeliverEvent() {
    throw new Error('no bindings');
  }
}

export function makeTarget() {
  return new Rows();
}

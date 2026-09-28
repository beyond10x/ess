// The #167/#175 desk (`crates/specify/ess-compiler/tests/fixtures/set-effects.yaml`), the target
// `tests/set_effects.rs` implements in Rust. Modes: `correct`, `end-ignores-filter`,
// `end-skips-one`, `end-miscounts`, `end-ignores-from`, `end-ignores-sets`, `note-ignores-filter`,
// `note-skips-one`, `note-miscounts`, `invite-holds-subject`, `invite-skips-others`,
// `invite-holds-every-team`.
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const text = (input, field) => (typeof input[field] === 'string' ? input[field] : '');
const took = (outcome, event, payload) => ({
  outcome,
  consistency: 'write',
  directEvents: [{ event, payload }],
});

class Desk {
  rows = [];
  next = 0;
  identity() {
    return { name: 'set-effects', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  single(id, outcome, event, to) {
    const row = this.rows.find((held) => held.session_id === id);
    if (row === undefined || row.state !== 'Open') {
      return { outcome: 'not-open', error: 'demo.desk.NotOpen', consistency: 'write' };
    }
    row.state = to;
    return took(outcome, event, { session_id: id });
  }
  executeCommand({ command, input }) {
    switch (command) {
      case 'demo.desk.Open': {
        this.next += 1;
        const id = `session-${this.next}`;
        this.rows.push({
          session_id: id,
          team: text(input, 'team'),
          note: text(input, 'note'),
          on_hold: false,
          state: 'Open',
        });
        return took('opened', 'demo.desk.SessionOpened', { session_id: id, team: text(input, 'team') });
      }
      case 'demo.desk.Park':
        return this.single(text(input, 'session_id'), 'parked', 'demo.desk.SessionParked', 'Parked');
      case 'demo.desk.Close':
        return this.single(text(input, 'session_id'), 'closed', 'demo.desk.SessionClosed', 'Ended');
      case 'demo.desk.EndTeam': {
        const team = text(input, 'team');
        let ended = 0;
        for (const row of this.rows) {
          const matches = mode === 'end-ignores-filter' || row.team === team;
          const movable = row.state === 'Open' || (mode === 'end-ignores-from' && row.state === 'Parked');
          if (!(matches && movable) || (mode === 'end-skips-one' && ended === 1)) {
            continue;
          }
          row.state = 'Ended';
          if (mode !== 'end-ignores-sets') {
            row.note = text(input, 'note');
          }
          ended += 1;
        }
        if (mode === 'end-miscounts') {
          ended += 1;
        }
        return took('ended', 'demo.desk.TeamEnded', { team, ended });
      }
      case 'demo.desk.NoteTeam': {
        const team = text(input, 'team');
        let noted = 0;
        for (const row of this.rows) {
          const matches = mode === 'note-ignores-filter' || row.team === team;
          if (!matches || (mode === 'note-skips-one' && noted === 1)) {
            continue;
          }
          row.note = text(input, 'note');
          noted += 1;
        }
        if (mode === 'note-miscounts') {
          noted = 0;
        }
        return took('noted', 'demo.desk.TeamNoted', { team, noted });
      }
      case 'demo.desk.Invite': {
        const id = text(input, 'session_id');
        const subject = this.rows.find((row) => row.session_id === id);
        if (subject === undefined) {
          return {};
        }
        for (const row of this.rows) {
          if (row.session_id === id) {
            row.on_hold = mode === 'invite-holds-subject';
            continue;
          }
          const selected = mode === 'invite-holds-every-team' || row.team === subject.team;
          if (selected && mode !== 'invite-skips-others') {
            row.on_hold = true;
          }
        }
        return took('invited', 'demo.desk.Invited', { session_id: id });
      }
      default:
        throw new Error(`unexpected command ${command}`);
    }
  }
  queryView({ view }) {
    if (view !== 'demo.desk.SessionDetails') {
      throw new Error(`unexpected view ${view}`);
    }
    return { rows: this.rows.map((row) => ({ ...row })) };
  }
  configureExternalOutcome() {
    throw unsupported('the model declares none');
  }
  redeliverEvent() {
    throw unsupported('unused');
  }
  observeEvents() {
    throw unsupported('unused');
  }
}

export function makeTarget() {
  return new Desk();
}

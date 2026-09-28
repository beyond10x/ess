// The #168 notes model of `tests/caller_values.rs`: a note records its caller's account and agent,
// and only its agent may edit it. Modes: `correct`, `first-account-ever`, `anyone-edits`,
// `only-the-first-caller-edits`, `cannot-authenticate`. The first caller is remembered for the whole
// run, across scenarios, as the Rust target remembers it.
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;
let first;
const same = (left, right) => JSON.stringify(left) === JSON.stringify(right);

class Notes {
  notes = new Map();
  identity() {
    return { name: 'notes-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input, caller }) {
    if (caller === undefined) {
      throw new Error('every command of this system is sent as a caller');
    }
    if (mode === 'cannot-authenticate') {
      throw unsupported('this target holds one credential');
    }
    first ??= caller;
    minted += 1;
    const consistency = `seq:${minted}`;
    if (command === 'demo.notes.CreateNote') {
      const id = `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
      const account = mode === 'first-account-ever' ? first.account_id : caller.account_id;
      this.notes.set(id, {
        note_id: id,
        account_id: account,
        agent_id: caller.agent_id,
        text: input.text,
        state: 'Open',
      });
      return {
        outcome: 'created',
        consistency,
        directEvents: [
          {
            event: 'demo.notes.NoteCreated',
            payload: { note_id: id, account_id: account, text: input.text },
          },
        ],
      };
    }
    if (command === 'demo.notes.EditNote') {
      const row = typeof input.note_id === 'string' ? this.notes.get(input.note_id) : undefined;
      if (row === undefined) {
        return { consistency };
      }
      const refused =
        mode === 'anyone-edits'
          ? false
          : mode === 'only-the-first-caller-edits'
            ? !same(caller, first)
            : row.agent_id !== caller.agent_id;
      if (refused) {
        return { outcome: 'forbidden', error: 'demo.notes.NotYourNote', consistency };
      }
      row.text = input.text;
      return {
        outcome: 'edited',
        consistency,
        directEvents: [
          { event: 'demo.notes.NoteEdited', payload: { note_id: input.note_id, text: input.text } },
        ],
      };
    }
    throw new Error(`unexpected command ${command}`);
  }
  queryView() {
    return {
      rows: [...this.notes.entries()]
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([, row]) => row),
    };
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
  return new Notes();
}

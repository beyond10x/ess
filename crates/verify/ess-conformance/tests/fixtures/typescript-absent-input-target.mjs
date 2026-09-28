// The #170 notes model (`crates/specify/ess-compiler/tests/fixtures/absent-input.yaml`), the target
// `tests/absent_input.rs` implements in Rust. Modes: `correct`, `reads-absent-as-empty`,
// `accepts-absent`, `cannot-omit` (the method throws unsupported), `no-method` (the target does not
// define it at all).
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let sequence = 0;

class Notes {
  rows = new Map();
  identity() {
    return { name: 'absent-input', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  took(outcome) {
    return { outcome, consistency: 'write', directEvents: [] };
  }
  submit(text) {
    sequence += 1;
    const id = `00000000-0000-4000-8000-${String(sequence).padStart(12, '0')}`;
    this.rows.set(id, typeof text === 'string' ? text : '');
    const result = this.took('submitted');
    result.directEvents.push({ event: 'demo.notes.NoteSubmitted', payload: { note_id: id } });
    return result;
  }
  executeCommand({ command, input }) {
    if (command === 'demo.notes.SubmitNote') {
      // `{}`: present, lacking a field — not the declared absent-body answer.
      return Object.hasOwn(input, 'text') ? this.submit(input.text) : {};
    }
    if (command === 'demo.notes.RewordNote') {
      const id = typeof input.note_id === 'string' ? input.note_id : '';
      if (!this.rows.has(id)) {
        return {};
      }
      this.rows.set(id, typeof input.text === 'string' ? input.text : '');
      const result = this.took('reworded');
      result.directEvents.push({ event: 'demo.notes.NoteReworded', payload: { note_id: id } });
      return result;
    }
    throw new Error(`unexpected command ${command}`);
  }
  queryView() {
    return { rows: [...this.rows.keys()].sort().map((id) => ({ note_id: id })) };
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

class WithoutInput extends Notes {
  executeCommandWithoutInput(request) {
    switch (mode) {
      case 'reads-absent-as-empty':
        return this.executeCommand({ ...request, input: {} });
      case 'accepts-absent':
        return this.submit('');
      case 'cannot-omit':
        throw unsupported('this adapter always sends a body');
      default:
        return { ...this.took('body-missing'), error: 'demo.notes.BodyMissing' };
    }
  }
}

export function makeTarget() {
  return mode === 'no-method' ? new Notes() : new WithoutInput();
}

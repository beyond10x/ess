// The #139 presence model: `partner_ref` is `null_when_absent`, `discount_code`
// `omitted_when_absent`, `note` declares neither, and `receipt.code` is `null_when_absent` inside a
// struct that is always there. Modes: `correct`, `omits-partner-ref`, `nulls-discount-code`,
// `omits-receipt-code`.
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const has = (object, key) =>
  object !== null && typeof object === 'object' && Object.hasOwn(object, key);
const absent = (object, key) => !has(object, key) || object[key] === null;

class Orders {
  identity() {
    return { name: 'presence-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'demo.orders.Place') {
      throw new Error(`unexpected command ${command}`);
    }
    const payload = {};
    if (!absent(input, 'partner_ref')) {
      payload.partner_ref = input.partner_ref;
    } else if (mode !== 'omits-partner-ref') {
      payload.partner_ref = null;
    }
    if (!absent(input, 'discount_code')) {
      payload.discount_code = input.discount_code;
    } else if (mode === 'nulls-discount-code') {
      payload.discount_code = null;
    }
    if (has(input, 'note')) {
      payload.note = input.note;
    }
    const receipt = input.receipt ?? {};
    payload.receipt = {};
    if (!absent(receipt, 'code')) {
      payload.receipt.code = receipt.code;
    } else if (mode !== 'omits-receipt-code') {
      payload.receipt.code = null;
    }
    if (has(input, 'later')) {
      payload.later = input.later;
    }
    return {
      outcome: 'placed',
      consistency: 'seq:1',
      directEvents: [{ event: 'demo.orders.Placed', payload }],
    };
  }
  queryView() {
    return { rows: [] };
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
  return new Orders();
}

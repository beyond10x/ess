// generated from billing v3
// model digest 8b52fe739078f96a006d7bee5e9b9530c3a30221f7bc003f291dcfe17cdcfea3
// contract digest 0c2f2067136aea0bc0a45ca5b01bf70f551fc6199956699c5d5b939c350688f8
// do not edit: regenerate with `ess synthesize --target web`

// The boundary, written once. The page imports it, and so does the smoke test that loads the
// module outside a browser — so what the test exercises is the page's own glue and not a second
// implementation that happens to agree with it.
//
// Three exports and nothing else. `memory.buffer` is read afresh on every access because
// allocating inside the module may grow the memory, which detaches every view taken before it.

/**
 * Instantiates a module and answers a driver over it.
 *
 * `source` is anything `WebAssembly.instantiate` accepts: the bytes of a `.wasm`, or a compiled
 * `Module`. If the module exports the optional realization hook, it is called once, before any
 * request — that is how a host that linked implementations of the plan's obligations reaches a
 * page neither of them was written against.
 */
export async function open(source) {
  const { instance } = await WebAssembly.instantiate(source, {});
  const exports = instance.exports;
  for (const name of EXPORTS) {
    if (typeof exports[name] !== "function") {
      throw new Error(`the module does not export ${name}; the page and the module disagree`);
    }
  }
  const realized = typeof exports[REALIZE] === "function";
  if (realized) exports[REALIZE]();

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  function request(body) {
    const bytes = encoder.encode(JSON.stringify(body));
    const address = exports.ess_input_reserve(bytes.length);
    new Uint8Array(exports.memory.buffer, address, bytes.length).set(bytes);
    const at = exports.ess_dispatch();
    const length = exports.ess_output_len();
    const text = decoder.decode(new Uint8Array(exports.memory.buffer, at, length));
    return JSON.parse(text);
  }

  return { request, realized, exports };
}

export const EXPORTS = ["ess_input_reserve", "ess_dispatch", "ess_output_len"];

export const REALIZE = "ess_realize";

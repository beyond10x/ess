//! The plain `httpAdapter` reads the refusal a command's failed answer declares, so the
//! `ess ui test --playwright` spec's 422 answer shows the refusal's error, not a bare status.
mod support;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = r"format: ess-ui/1
app: refusals
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: Refusals
    sections:
      - {name: list, component: collection, reads: {view: t.All}, columns: [id]}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-plain-refusal")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    ess_ui_react::generate(&ess_ui::load_str(DOCUMENT).unwrap(), Path::new("."), &out).unwrap();
    out
}

#[test]
fn a_failed_command_with_a_declared_refusal_body_fails_with_the_refusals_error() {
    let out = project("send");
    support::compile(&out, &["runtime/data.ts"]);
    support::node(
        &out,
        "send",
        r"
(async () => {
  const {httpAdapter, DataError} = out('runtime/data');
  const answers = {
    refused: {status: 422, statusText: 'Unprocessable Entity', body: JSON.stringify({outcome: 'refused', published: [], error: 'already_cancelled', payload: {n: 1}})},
    other: {status: 500, statusText: 'Internal Server Error', body: 'boom'},
    json: {status: 400, statusText: 'Bad Request', body: JSON.stringify({message: 'no'})},
    ok: {status: 200, statusText: 'OK', body: JSON.stringify({accepted: true})},
  };
  global.fetch = async (url) => {
    const found = answers[url.split('/commands/')[1]];
    return {ok: found.status < 300, status: found.status, statusText: found.statusText, text: async () => found.body, json: async () => JSON.parse(found.body)};
  };
  const adapter = httpAdapter('http://app.test');
  const failure = async (command) => adapter.send(command, {}).then(() => assert.fail('accepted'), (error) => error);
  const refused = await failure('refused');
  assert.ok(refused instanceof DataError);
  assert.equal(refused.message, 'already_cancelled');
  assert.equal(refused.status, 422);
  const other = await failure('other');
  assert.ok(other instanceof DataError);
  assert.equal(other.message, '500 Internal Server Error');
  assert.equal((await failure('json')).message, '400 Bad Request');
  assert.deepEqual(await adapter.send('ok', {}), {accepted: true});
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

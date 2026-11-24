//! A `multipart/form-data` request (RFC 7578) sent to the generated Rust server (beyond10x/ess#754,
//! `story:the-generated-rust-server-reads-a-multipart-request`): a text part is read as a `form`
//! value, a file part's bytes as the input's `Bytes`, its filename and media type into the input
//! fields `parts` names, and a request out of the rules is `malformed`, naming the part.
//!
//! A request line holds no line break, so a body spells each CRLF as the four characters `\r\n`,
//! which the harness turns back into the two bytes before it sends the body.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{ir_of, rust_run_with};

/// The shop gains a `Format` enum and three commands taking files, each emitting an event that
/// carries every input field, so the answer's envelope shows what the server read.
fn model() -> String {
    let edits = [
        (
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n",
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - {name: shop.items.Format, kind: enum, variants: [csv, xlsx]}
",
        ),
        (
            "events:\n  - name: shop.items.Put\n",
            "events:
  - name: shop.items.Imported
    fields:
      - {name: file, type: Bytes}
      - {name: file_name, type: String}
      - {name: format, type: shop.items.Format}
      - {name: dry_run, type: Boolean}
  - name: shop.items.Photographed
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: photo, type: Optional<Bytes>}
      - {name: photo_name, type: Optional<String>}
      - {name: photo_type, type: Optional<String>}
      - {name: caption, type: String}
      - {name: tags, type: List<String>}
  - name: shop.items.Filed
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: files, type: List<Bytes>}
  - name: shop.items.Put\n",
        ),
        (
            "  - name: shop.items.Purge\n",
            "  - name: shop.items.Import
    input:
      - {name: file, type: Bytes}
      - {name: file_name, type: String}
      - {name: format, type: shop.items.Format}
      - {name: dry_run, type: Boolean}
    outcomes:
      - name: imported
        emits: [shop.items.Imported]
        payload:
          shop.items.Imported: {file: input.file, file_name: input.file_name, format: input.format, dry_run: input.dry_run}
  - name: shop.items.AttachPhoto
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: photo, type: Optional<Bytes>}
      - {name: photo_name, type: Optional<String>}
      - {name: photo_type, type: Optional<String>}
      - {name: caption, type: String}
      - {name: tags, type: List<String>}
    outcomes:
      - name: photographed
        emits: [shop.items.Photographed]
        payload:
          shop.items.Photographed: {item_id: input.item_id, photo: input.photo, photo_name: input.photo_name, photo_type: input.photo_type, caption: input.caption, tags: input.tags}
  - name: shop.items.AttachFiles
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: files, type: List<Bytes>}
    outcomes:
      - name: filed
        emits: [shop.items.Filed]
        payload:
          shop.items.Filed: {item_id: input.item_id, files: input.files}
  - name: shop.items.Purge\n",
        ),
        (
            "      - shop.items.Approve\ncomponents:\n",
            "      - shop.items.Approve
      - shop.items.Import
      - shop.items.AttachPhoto
      - shop.items.AttachFiles
components:\n",
        ),
        (
            "        - shop.items.Archive\n",
            "        - shop.items.Archive
        - shop.items.Import
        - shop.items.AttachPhoto
        - shop.items.AttachFiles\n",
        ),
        (
            "shop.items.Searched, shop.items.Approved]",
            "shop.items.Searched, shop.items.Approved, shop.items.Imported, shop.items.Photographed, shop.items.Filed]",
        ),
    ];
    let mut model = shared::MODEL.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            model.matches(anchor).count(),
            1,
            "the model moved: {anchor}"
        );
        model = model.replace(anchor, replacement);
    }
    model
}

const ROUTES: &str = "routes:
  - command: shop.items.Import
    method: post
    path: /imports
    request:
      encoding: multipart
      parts:
        file: {filename: input.file_name, media_types: [text/csv]}
  - command: shop.items.AttachPhoto
    method: post
    path: /photos/{item_id}
    parameters: {item_id: input.item_id}
    request:
      encoding: multipart
      unknown: ignore
      parts:
        photo:
          filename: input.photo_name
          media_type: input.photo_type
          media_types: [image/png, image/jpeg]
  - command: shop.items.AttachFiles
    method: post
    path: /files/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: multipart}
";

const TYPE: &str = "multipart/form-data; boundary=XyZ";

/// The story's request: a `file` part `rows.csv` of `text/csv` holding `a;b` and a line break, a
/// `format` part `csv` and a `dry_run` part `true`.
const IMPORT: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.csv"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na;b\r\n\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="format"\r\n\r\ncsv\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n--XyZ--\r\n"#,
);

/// [`IMPORT`] less its closing boundary.
const UNCLOSED: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.csv"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na;b\r\n"#,
);

const WRONG_TYPE: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.txt"\r\n"#,
    r#"Content-Type: text/plain\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="format"\r\n\r\ncsv\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n--XyZ--"#,
);

const TWICE: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.csv"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="format"\r\n\r\ncsv\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\nfalse\r\n--XyZ--"#,
);

const UNKNOWN: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.csv"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="format"\r\n\r\ncsv\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="zzz"\r\n\r\n1\r\n--XyZ--"#,
);

const NO_FILENAME: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="format"\r\n\r\ncsv\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n--XyZ--"#,
);

const NO_FORMAT: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="file"; filename="rows.csv"\r\n"#,
    r#"Content-Type: text/csv\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="dry_run"\r\n\r\ntrue\r\n--XyZ--"#,
);

/// A photo with its name and media type, two `tags` parts, a part no field names and an echo of
/// the path's `item_id`, all under `unknown: ignore`.
const PHOTO: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="photo"; filename="a.png"\r\n"#,
    r#"Content-Type: image/png\r\n\r\nPNG\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="caption"\r\n\r\nhi\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="tags"\r\n\r\na\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="tags"\r\n\r\nb\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="origin"\r\n\r\nweb\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="item_id"\r\n\r\np1\r\n--XyZ--"#,
);

const PHOTO_OTHER: &str = concat!(
    r#"--XyZ\r\nContent-Disposition: form-data; name="caption"\r\n\r\nhi\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="item_id"\r\n\r\np2\r\n--XyZ--"#,
);

/// No photo, which is `Optional`, and no tags, an empty list.
const NO_PHOTO: &str =
    r#"--XyZ\r\nContent-Disposition: form-data; name="caption"\r\n\r\nhi\r\n--XyZ--"#;

/// Two `files` parts between a preamble and an epilogue, which are ignored.
const FILES: &str = concat!(
    r#"preamble\r\n"#,
    r#"--XyZ\r\nContent-Disposition: form-data; name="files"; filename="1"\r\n\r\none\r\n"#,
    r#"--XyZ  \r\nContent-Disposition: form-data; name="files"; filename="2"\r\n\r\ntwo\r\n"#,
    r#"--XyZ--\r\nepilogue"#,
);

const REQUESTS: &[[&str; 7]] = &[
    ["import", "y", "POST", "/imports", "", TYPE, IMPORT],
    [
        "import-json-type",
        "y",
        "POST",
        "/imports",
        "",
        "application/json",
        IMPORT,
    ],
    [
        "import-no-boundary",
        "y",
        "POST",
        "/imports",
        "",
        "multipart/form-data",
        IMPORT,
    ],
    [
        "import-quoted-boundary",
        "y",
        "POST",
        "/imports",
        "",
        r#"Multipart/Form-Data; charset=utf-8; boundary="XyZ""#,
        IMPORT,
    ],
    [
        "import-unclosed",
        "y",
        "POST",
        "/imports",
        "",
        TYPE,
        UNCLOSED,
    ],
    [
        "import-wrong-type",
        "y",
        "POST",
        "/imports",
        "",
        TYPE,
        WRONG_TYPE,
    ],
    ["import-twice", "y", "POST", "/imports", "", TYPE, TWICE],
    ["import-unknown", "y", "POST", "/imports", "", TYPE, UNKNOWN],
    [
        "import-no-filename",
        "y",
        "POST",
        "/imports",
        "",
        TYPE,
        NO_FILENAME,
    ],
    [
        "import-no-format",
        "y",
        "POST",
        "/imports",
        "",
        TYPE,
        NO_FORMAT,
    ],
    ["photo", "y", "POST", "/photos/p1", "", TYPE, PHOTO],
    [
        "photo-other",
        "y",
        "POST",
        "/photos/p1",
        "",
        TYPE,
        PHOTO_OTHER,
    ],
    [
        "photo-absent",
        "y",
        "POST",
        "/photos/p1",
        "",
        TYPE,
        NO_PHOTO,
    ],
    ["files", "y", "POST", "/files/f1", "", TYPE, FILES],
];

/// The Rust harness, sending each body with its `\r\n` spelled back as CRLF.
fn rust_harness() -> String {
    let body = "body: body.as_bytes().to_vec(),";
    assert!(
        shared::RUST_HARNESS.contains(body),
        "the harness sends a body"
    );
    shared::RUST_HARNESS.replace(
        body,
        "body: body.replace(\"\\\\r\\\\n\", \"\\r\\n\").into_bytes(),",
    )
}

/// The Rust server's answers to [`REQUESTS`], once per test binary.
fn rust() -> &'static BTreeMap<String, Answer> {
    static RUST: std::sync::OnceLock<BTreeMap<String, Answer>> = std::sync::OnceLock::new();
    RUST.get_or_init(|| {
        let ir = ir_of(&model());
        let bound = shared::binding(&ir, ROUTES);
        let synthesis: Synthesis =
            synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        rust_run_with("w754multipart", &synthesis, REQUESTS, &rust_harness())
    })
}

fn answered(label: &str) -> &'static Answer {
    rust()
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"))
}

/// `label` answered the default `malformed`, naming `detail`.
fn malformed(label: &str, detail: &str) {
    let answer = answered(label);
    assert_eq!(answer.status, 400, "{label}: {answer:?}");
    assert!(answer.body.contains(detail), "{label}: {answer:?}");
}

/// `label` ran its command, and its answer holds each of `members`.
fn ran(label: &str, members: &[&str]) {
    let answer = answered(label);
    assert!(answer.status < 300, "{label}: {answer:?}");
    for member in members {
        assert!(
            answer.body.contains(member),
            "{label}: {member} in {answer:?}"
        );
    }
}

#[test]
fn a_multipart_request_runs_the_command_with_its_parts() {
    // `a;b` and CRLF, as base64.
    let import = [
        r#""file":"YTtiDQo=""#,
        r#""file_name":"rows.csv""#,
        r#""format":"csv""#,
        r#""dry_run":true"#,
    ];
    ran("import", &import);
    ran("import-quoted-boundary", &import);
    ran(
        "photo",
        &[
            r#""photo":"UE5H""#,
            r#""photo_name":"a.png""#,
            r#""photo_type":"image/png""#,
            r#""caption":"hi""#,
            r#""tags":["a","b"]"#,
            r#""item_id":"p1""#,
        ],
    );
    ran("photo-absent", &[r#""caption":"hi""#, r#""tags":[]"#]);
    assert!(
        !answered("photo-absent").body.contains("photo_name"),
        "{:?}",
        answered("photo-absent")
    );
    ran("files", &[r#""files":["b25l","dHdv"]"#]);
}

#[test]
fn a_multipart_request_out_of_its_rules_is_malformed_naming_the_part() {
    malformed("import-json-type", "the body is not `multipart/form-data`");
    malformed(
        "import-no-boundary",
        "the body is `multipart/form-data` without a `boundary`",
    );
    malformed(
        "import-unclosed",
        "the body is not closed by its final boundary",
    );
    malformed(
        "import-wrong-type",
        "body.file: the part's media type is `text/plain`, not one of `text/csv`",
    );
    malformed("import-twice", "body.dry_run: expected one part, found 2");
    malformed(
        "import-unknown",
        "body.zzz: no field of this request is named so",
    );
    malformed(
        "import-no-filename",
        "body.file: the part has no filename, which input field `file_name` requires",
    );
    malformed("import-no-format", "format");
    malformed(
        "photo-other",
        "body.item_id: the path carries this field with another value",
    );
}

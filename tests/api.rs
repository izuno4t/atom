use std::io::Cursor;

use anything_to_markdown::{ConversionOptions, Flavor, convert_bytes, convert_reader};

#[test]
fn library_api_converts_bytes_and_readers() {
    let options = ConversionOptions {
        flavor: Flavor::Gfm,
        ..ConversionOptions::default()
    };

    let from_bytes = convert_bytes("sample.html", b"<h1>Title</h1>", options.clone()).unwrap();
    let from_reader = convert_reader(
        "sample.html",
        Cursor::new(b"<h1>Title</h1>".to_vec()),
        options,
    )
    .unwrap();

    assert_eq!(from_bytes.markdown, "# Title\n");
    assert_eq!(from_reader.markdown, "# Title\n");
}

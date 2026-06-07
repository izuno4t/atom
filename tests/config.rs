use std::fs;
use std::path::Path;

use anything_to_markdown::{Flavor, LlmBackend, OcrEngine, load_config};

#[test]
fn atom_config_toml_loads_core_conversion_options() {
    let path = Path::new("target/config-test/atom.config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
flavor = "gfm"
ocr = "ndlocr-lite"
llm = "ollama:llama3"
translate = "ja"
extract_media = "media"
inline_base64_media = true
"#,
    )
    .unwrap();

    let options = load_config(path).unwrap();

    assert_eq!(options.flavor, Flavor::Gfm);
    assert_eq!(options.ocr, OcrEngine::NdlOcrLite);
    assert_eq!(options.llm, LlmBackend::Ollama("llama3".to_string()));
    assert_eq!(options.translate, Some("ja".to_string()));
    assert_eq!(options.extract_media.unwrap(), Path::new("media"));
    assert!(options.inline_base64_media);
}

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use anything_to_markdown::{
    ConversionOptions, Flavor, LlmBackend, OcrEngine, apply_config, apply_user_config, load_config,
    parse_llm, user_config_paths,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn atom_config_toml_loads_core_conversion_options() {
    let path = Path::new("target/config-test/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path.parent().unwrap().join("restructure.prompt.txt"),
        "Restructure only: {input}",
    )
    .unwrap();
    fs::write(
        path,
        r#"
flavor = "gfm"
ocr = "ndlocr-lite"
llm = "ollama:llama3"
translate = "ja"
extract_media = "media"
inline_base64_media = true
llm.prompt_path.restructure = "restructure.prompt.txt"
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
    assert_eq!(
        options.llm_prompts.get("restructure").map(String::as_str),
        Some("Restructure only: {input}")
    );
}

#[test]
fn parse_llm_accepts_openai_compatible_endpoint() {
    assert_eq!(
        parse_llm("openai-compatible:local@https://llm.example.test/v1"),
        LlmBackend::OpenAiCompatible {
            name: "local".to_string(),
            endpoint: "https://llm.example.test/v1".to_string(),
        }
    );
}

#[test]
fn parse_llm_accepts_gemini_models() {
    assert_eq!(
        parse_llm("gemini:gemini-2.5-flash"),
        LlmBackend::Gemini("gemini-2.5-flash".to_string())
    );
    assert_eq!(
        parse_llm("gemini-2.5-pro"),
        LlmBackend::Gemini("gemini-2.5-pro".to_string())
    );
}

#[test]
fn apply_config_overlays_existing_options_without_resetting_cli_like_values() {
    let path = Path::new("target/config-test/overlay.config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path.parent().unwrap().join("translate.prompt.txt"),
        "Translate into {language}: {input}",
    )
    .unwrap();
    fs::write(
        path,
        r#"
llm = "ollama:qwen"
llm.prompt_path.translate = "translate.prompt.txt"
"#,
    )
    .unwrap();
    let mut options = ConversionOptions {
        restructure: true,
        strict: true,
        ..Default::default()
    };

    apply_config(&mut options, path).unwrap();

    assert!(options.restructure);
    assert!(options.strict);
    assert_eq!(options.llm, LlmBackend::Ollama("qwen".to_string()));
    assert_eq!(
        options.llm_prompts.get("translate").map(String::as_str),
        Some("Translate into {language}: {input}")
    );
}

#[test]
fn user_config_paths_uses_atom_home_directory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = Path::new("target/config-test/atom-home");
    unsafe {
        std::env::set_var("ATOM_HOME", root);
    }

    let paths = user_config_paths();

    assert_eq!(paths, vec![root.join("config.toml")]);
}

#[test]
fn apply_user_config_reads_first_existing_atom_home_config() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = Path::new("target/config-test/user-config");
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("image.prompt.txt"), "Describe image: {input}").unwrap();
    fs::write(
        root.join("config.toml"),
        r#"
llm = "ollama:gemma"
llm_prompt_path_image_description = "image.prompt.txt"
"#,
    )
    .unwrap();
    unsafe {
        std::env::set_var("ATOM_HOME", root);
    }
    let mut options = ConversionOptions::default();

    let loaded = apply_user_config(&mut options).unwrap();

    assert_eq!(loaded, Some(root.join("config.toml")));
    assert_eq!(options.llm, LlmBackend::Ollama("gemma".to_string()));
    assert_eq!(
        options
            .llm_prompts
            .get("image-description")
            .map(String::as_str),
        Some("Describe image: {input}")
    );
}

use anything_to_markdown::wasm_plugin::{ATOM_WASM_PLUGIN_ABI_VERSION, WasmPluginManifest};

#[test]
fn wasm_plugin_manifest_defaults_to_current_abi_and_entrypoint() {
    let manifest = WasmPluginManifest::new("normalize-headings", "0.1.0");

    assert_eq!(manifest.abi_version, ATOM_WASM_PLUGIN_ABI_VERSION);
    assert_eq!(manifest.entrypoint, "atom_filter");
    assert!(manifest.is_compatible());
}

#[test]
fn wasm_plugin_manifest_rejects_incompatible_abi() {
    let mut manifest = WasmPluginManifest::new("normalize-headings", "0.1.0");
    manifest.abi_version = ATOM_WASM_PLUGIN_ABI_VERSION + 1;

    assert!(!manifest.is_compatible());
}

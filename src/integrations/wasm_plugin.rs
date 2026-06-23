use crate::AstNode;

pub const ATOM_WASM_PLUGIN_ABI_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmPluginManifest {
    pub name: String,
    pub version: String,
    pub abi_version: u32,
    pub entrypoint: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WasmFilterRequest {
    pub input_format: String,
    pub nodes: Vec<AstNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WasmFilterResponse {
    pub nodes: Vec<AstNode>,
    pub warnings: Vec<String>,
}

impl WasmPluginManifest {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            abi_version: ATOM_WASM_PLUGIN_ABI_VERSION,
            entrypoint: "atom_filter".to_string(),
        }
    }

    pub fn is_compatible(&self) -> bool {
        self.abi_version == ATOM_WASM_PLUGIN_ABI_VERSION && !self.entrypoint.trim().is_empty()
    }
}

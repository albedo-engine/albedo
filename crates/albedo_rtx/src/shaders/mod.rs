use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "shaders/imports"]
#[prefix = "imports/"]
pub struct AlbedoRtxShaderImports;

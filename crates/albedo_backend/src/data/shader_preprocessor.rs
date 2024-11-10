use std::{collections::HashMap, fmt::Debug};
use wgpu::naga;

pub enum ShaderError {
    SyntaxError,
    Missing(String)
}

#[derive(Debug)]
pub enum CompileError {
    Preprocessor(ShaderError),
    Module(naga::front::glsl::ParseErrors),
}

impl From<naga::front::glsl::ParseErrors> for CompileError {
    fn from(value: naga::front::glsl::ParseErrors) -> Self {
        CompileError::Module(value)
    }
}

impl From<ShaderError> for CompileError {
    fn from(value: ShaderError) -> Self {
        CompileError::Preprocessor(value)
    }
}

impl Debug for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError => write!(f, "SyntaxError"),
            Self::Missing(import) => write!(f, "Missing import: '{}'", import),
        }
    }
}

pub struct ShaderCache {
    imports: HashMap<String, String>,
}

impl ShaderCache {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    pub fn add_embedded<T: rust_embed::Embed>(&mut self) {
        for file in T::iter() {
            let contents = String::from_utf8(T::get(&file).unwrap().data.into_owned()).unwrap();
            self.imports.insert(file.to_string(), contents);
        }
    }

    pub fn compile(&self, source: &str) -> Result<String, ShaderError> {
        let lines = source.lines();
        let mut buf = String::new();
        for line in lines {
            let line: &str = line.trim();
            let Some(start) = line.find("#include") else {
                buf.extend([line, "\n"]);
                continue;
            };
            let include = line[(start + ("#include".len())..)].trim();
            if !include.starts_with("\"") {
                return Err(ShaderError::SyntaxError);
            }
            let Some(end) = include[1..].find("\"") else {
                return Err(ShaderError::SyntaxError);
            };
            let name = &include[1..end+1];
            let Some(content) = self.imports.get(name) else {
                return Err(ShaderError::Missing(name.to_string()));
            };
            buf.extend([content, "\n"]);
        }
        Ok(buf)
    }

    pub fn compile_compute(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, naga::ShaderStage::Compute)
    }
    pub fn compile_fragment(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, naga::ShaderStage::Fragment)
    }
    pub fn compile_vertex(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, naga::ShaderStage::Vertex)
    }

    pub fn compile_module(&self, source: &str, stage: naga::ShaderStage) -> Result<naga::Module, CompileError> {
        let source = self.compile(source)?;
        let module = naga::front::glsl::Frontend::default()
            .parse(
                &naga::front::glsl::Options {
                    stage,
                    defines: Default::default(),
                },
                &source,
            )?;
        Ok(module)
    }
}

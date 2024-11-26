use std::{collections::HashMap, fmt::Debug, path::Path};
use wgpu::naga::{self, FastHashMap};

pub enum PreprocessError {
    SyntaxError,
    Missing(String)
}

#[derive(Debug)]
pub enum CompileError {
    Preprocessor(PreprocessError),
    Module(naga::front::glsl::ParseErrors),
}

impl From<naga::front::glsl::ParseErrors> for CompileError {
    fn from(value: naga::front::glsl::ParseErrors) -> Self {
        CompileError::Module(value)
    }
}

impl From<PreprocessError> for CompileError {
    fn from(value: PreprocessError) -> Self {
        CompileError::Preprocessor(value)
    }
}

impl Debug for PreprocessError {
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

    pub fn add_path(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let name = path.file_name().unwrap();
        self.add_raw(name.to_str().unwrap(), &content);
        Ok(())
    }

    pub fn add_raw(&mut self, name: &str, content: &str) {
        self.imports.insert(name.to_string(), content.to_string());
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, directory: P) -> Result<(), std::io::Error> {
        let paths = std::fs::read_dir(directory)?;
        for entry in paths {
            let Ok(entry) = entry else { continue; };
            let Ok(meta) = entry.metadata() else { continue; };
            if !meta.is_file() { continue; }

            let path = entry.path();
            let Some(ext) = path.extension().map(|s| s.to_str()).flatten() else {
                continue;
            };
            match ext {
                "comp"|"frag"|"vert" => (),
                _ => { continue; }
            }

            let Some(filename) = path.file_name().map(|s| s.to_str()).flatten() else { continue; };
            let content = std::fs::read_to_string(entry.path())?;
            self.imports.insert(filename.to_string(), content);
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.imports.get(name).map(|s| s.as_str())
    }

    pub fn compile(&self, source: &str) -> Result<String, PreprocessError> {
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
                return Err(PreprocessError::SyntaxError);
            }
            let Some(end) = include[1..].find("\"") else {
                return Err(PreprocessError::SyntaxError);
            };
            let name = &include[1..end+1];
            let Some(content) = self.imports.get(name) else {
                return Err(PreprocessError::Missing(name.to_string()));
            };
            buf.extend([content, "\n"]);
        }
        Ok(buf)
    }

    pub fn compile_compute(&self, source: &str, defines: Option<&FastHashMap<String, String>>) -> Result<naga::Module, CompileError> {
        let defines = match defines {
            Some(d) => d.clone(),
            None => FastHashMap::default()
        };
        self.compile_module(source, defines, naga::ShaderStage::Compute)
    }
    pub fn compile_fragment(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, FastHashMap::default(), naga::ShaderStage::Fragment)
    }
    pub fn compile_vertex(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, FastHashMap::default(), naga::ShaderStage::Vertex)
    }

    pub fn compile_module(&self, source: &str, defines: FastHashMap<String, String>, stage: naga::ShaderStage) -> Result<naga::Module, CompileError> {
        let source = self.compile(source)?;
        let module = naga::front::glsl::Frontend::default()
            .parse(
                &naga::front::glsl::Options {
                    stage,
                    defines
                },
                &source,
            )?;
        Ok(module)
    }
}

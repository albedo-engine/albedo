use std::{
    collections::HashMap,
    fmt::{format, Debug},
    ops::Range,
    path::Path,
};
use wgpu::naga::{self, FastHashMap, Span};

pub enum PreprocessError {
    SyntaxError,
    Missing(String),
}

#[derive(Debug)]
pub struct ParseError {
    pub import_name: Option<String>,
    pub line: u32,
    pub offset: u32,
    pub kind: naga::front::glsl::ErrorKind,
}

impl ParseError {
    pub fn new(kind: naga::front::glsl::ErrorKind) -> Self {
        Self {
            import_name: Default::default(),
            line: Default::default(),
            offset: Default::default(),
            kind,
        }
    }
}

#[derive(Debug)]
pub enum CompileError {
    Preprocessor(PreprocessError),
    Module(Vec<ParseError>),
}

impl From<Vec<ParseError>> for CompileError {
    fn from(value: Vec<ParseError>) -> Self {
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

struct InlinedResult {
    content: String,
    ranges: Vec<Range<u32>>,
    imports: Vec<String>,
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

    pub fn add_directory<P: AsRef<Path>>(
        &mut self,
        directory: P,
        prefix: Option<&str>,
    ) -> Result<(), std::io::Error> {
        let paths = std::fs::read_dir(directory)?;
        for entry in paths {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if !meta.is_file() {
                continue;
            }

            let path = entry.path();
            let Some(ext) = path.extension().map(|s| s.to_str()).flatten() else {
                continue;
            };
            match ext {
                "comp" | "frag" | "vert" | "glsl" => (),
                _ => {
                    continue;
                }
            }

            let Some(filename) = path.file_name().map(|s| s.to_str()).flatten() else {
                continue;
            };

            let content = std::fs::read_to_string(entry.path())?;
            match prefix.as_ref() {
                Some(p) => {
                    let filename = format!("{}/{}", p, filename);
                    self.imports.insert(filename.to_string(), content);
                }
                None => {
                    self.imports.insert(filename.to_string(), content);
                }
            };
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.imports.get(name).map(|s| s.as_str())
    }

    pub fn compile(&self, source: &str) -> Result<InlinedResult, PreprocessError> {
        let lines = source.lines();
        let mut buf = String::new();

        // TODO: Make optional to avoid extra processing when not needed.
        let mut ranges: Vec<Range<u32>> = Vec::new();
        let mut imports: Vec<String> = Vec::new();

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
            let name = &include[1..end + 1];
            let Some(content) = self.imports.get(name) else {
                return Err(PreprocessError::Missing(name.to_string()));
            };

            let start = buf.len();
            buf.extend([content, "\n"]);
            imports.push(name.into());
            ranges.push(start as u32..buf.len() as u32);
        }

        Ok(InlinedResult {
            content: buf,
            ranges,
            imports,
        })
    }

    pub fn compile_compute(
        &self,
        source: &str,
        defines: Option<&FastHashMap<String, String>>,
    ) -> Result<naga::Module, CompileError> {
        let defines = match defines {
            Some(d) => d.clone(),
            None => FastHashMap::default(),
        };
        self.compile_module(source, defines, naga::ShaderStage::Compute)
    }
    pub fn compile_fragment(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, FastHashMap::default(), naga::ShaderStage::Fragment)
    }
    pub fn compile_vertex(&self, source: &str) -> Result<naga::Module, CompileError> {
        self.compile_module(source, FastHashMap::default(), naga::ShaderStage::Vertex)
    }

    pub fn compile_module(
        &self,
        source: &str,
        defines: FastHashMap<String, String>,
        stage: naga::ShaderStage,
    ) -> Result<naga::Module, CompileError> {
        let source = self.compile(source)?;
        // TODO: Remap the error based on the include content.
        Ok(naga::front::glsl::Frontend::default()
            .parse(
                &naga::front::glsl::Options { stage, defines },
                &source.content,
            )
            .map_err(|e| {
                let errors = e
                    .errors
                    .into_iter()
                    .map(|error| {
                        let mut ret_error = ParseError::new(error.kind);
                        let Some(span) = error.meta.to_range() else {
                            return ret_error;
                        };

                        let Some(found) = source
                            .ranges
                            .iter()
                            .position(|r| r.contains(&(span.start as u32)))
                        else {
                            let loc = error.meta.location(&source.content);
                            ret_error.line = loc.line_number;
                            ret_error.offset = loc.line_position;
                            return ret_error;
                        };
                        let import_name = &source.imports[found];
                        let Some(import_content) = self.imports.get(import_name) else {
                            return ret_error;
                        };

                        let start_offset = source.ranges[found].start;
                        let span = Span::new(
                            span.start as u32 - start_offset,
                            span.start as u32 - start_offset,
                        );
                        let loc = span.location(&import_content);
                        ret_error.import_name = Some(import_name.clone());
                        ret_error.line = loc.line_number;
                        ret_error.offset = loc.line_position;
                        ret_error
                    })
                    .collect();
                CompileError::Module(errors)
            })?)
    }
}

use std::{collections::HashMap, fmt::Debug, str::Lines, usize};

pub enum ShaderError {
    SyntaxError,
    Missing
}

impl Debug for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError => write!(f, "SyntaxError"),
            Self::Missing => write!(f, "Missing"),
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

    pub fn compile(&mut self, source: &str) -> Result<String, ShaderError> {
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
                return Err(ShaderError::Missing);
            };
            buf.extend([content, "\n"]);
        }
        Ok(buf)
    }
}

#[cfg(not(windows))]
macro_rules! path_separator {
    ()=>{ "/" }
}

#[cfg(windows)]
macro_rules! path_separator {
    ()=>{r#"\"#}
}

pub(crate) use path_separator;

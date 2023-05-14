use std::{fs, io, path};

static INPUT_FOLDER: &str = "./src/shaders";
static OUTPUT_FOLDER: &str = "./src/shaders/spirv";

fn needs_compile<P1, P2>(input: P1, output: P2) -> io::Result<bool>
where
    P1: AsRef<path::Path>,
    P2: AsRef<path::Path>,
{
    let out_meta = fs::metadata(output);
    if let Ok(meta) = out_meta {
        let output_mtime = meta.modified()?;

        // if input file is more recent than our output, we are outdated
        let input_meta = fs::metadata(input)?;
        let input_mtime = input_meta.modified()?;

        Ok(input_mtime > output_mtime)
    } else {
        // output file not found, we are outdated.
        Ok(true)
    }
}

fn main() {
    // Ensures glslc is installed and available in PATH.
    if let Err(_) = std::process::Command::new("glslc")
        .args(&["--version"])
        .output()
    {
        panic!("glslc not found in PATH. Please install from: https://github.com/google/shaderc");
    }

    fs::create_dir(OUTPUT_FOLDER).ok();

    let shaders = fs::read_dir(INPUT_FOLDER).unwrap();

    for file in shaders {
        let path = file.unwrap().path();

        let filename = match path.file_name() {
            Some(f) => f,
            _ => continue,
        };
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("comp") | Some("frag") | Some("vert") => (),
            _ => continue,
        }

        let output_os =
            path::Path::new(OUTPUT_FOLDER).join(format!("{}.spv", filename.to_str().unwrap()));
        let output = output_os.to_str().unwrap();
        let input = path.to_str().unwrap();

        println!("cargo:rerun-if-changed={}", input);

        match needs_compile(input, output).unwrap_or(true) {
            false => continue,
            _ => (),
        }

        match std::process::Command::new("glslc")
            .args(&[input, "-o", output])
            .output()
        {
            Ok(o) => match o.status.success() {
                false => panic!(
                    "Failed to compile shader '{}':\n{}",
                    input,
                    String::from_utf8(o.stderr).unwrap()
                ),
                _ => (),
            },
            Err(_) => panic!("Failed to compile shader '{}'", input),
        }
    }
}

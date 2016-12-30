use std::path::Path;
use std::fs;
use std::io::{self, Write, Read};
use std::process::{Command, Stdio};

use regex::Regex;

use {Result, ResultExt};

#[derive(Debug)]
struct CrateImport {
    name: String,
    version: String,
    macro_use: bool,
}

impl CrateImport {
    pub fn new(name: &str, version: &str, macro_use: bool) -> Self {
        CrateImport {
            name: name.to_owned(),
            version: version.to_owned(),
            macro_use: macro_use,
        }
    }

    pub fn to_cargo_toml_format(&self) -> String {
        format!("{} = \"{}\"", self.name, self.version)
    }

    pub fn to_code_format(&self) -> String {
        let mut code = String::new();
        if self.macro_use {
            code.push_str("#[macro_use] ");
        }
        code.push_str(&format!("extern crate {};", self.name));
        code
    }
}

fn crates_to_code(crates: &[CrateImport]) -> String {
    crates.iter().fold(String::new(), |mut acc, crate_| {
        acc.push_str(&crate_.to_code_format());
        acc.push_str("\n");
        acc
    })
}

fn crates_to_toml_format(crates: &[CrateImport]) -> String {
    crates.iter().fold(String::new(), |mut acc, crate_| {
        acc.push_str(&crate_.to_cargo_toml_format());
        acc.push_str("\n");
        acc
    })
}

pub fn build_script_crate<P: AsRef<Path>, Q: AsRef<Path>>(script_path: P,
                                                          output_crate_dir: Q)
                                                          -> Result<()> {
    let script_path = script_path.as_ref();
    let script_name = script_path.file_name()
        .ok_or("Script has no name")?
        .to_str()
        .ok_or("Script name is not valid utf-8")?;
    let output_crate_dir = output_crate_dir.as_ref();
    let src_dir = output_crate_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir).chain_err(|| "unable to create src dir")?;
    }
    let main_path = src_dir.join("main.rs");

    let code = read_file(script_path)?;
    let (crates, code) = extract_extern_crates(code)?;

    let formatted_code = format_code(&crates, code);
    let mut output_f = fs::File::create(main_path).chain_err(|| "Unable to write main.rs")?;
    output_f.write_all(formatted_code.as_bytes()).chain_err(|| "Unable to write main.rs")?;

    create_toml(&script_name, &crates, &output_crate_dir)?;
    compile(output_crate_dir)
}

fn create_toml<P: AsRef<Path>>(script_name: &str,
                               extern_crates: &[CrateImport],
                               project_dir: P)
                               -> Result<()> {
    let project_dir = project_dir.as_ref();
    let toml = format!("[package]
        name = \"{}\"
        version = \"0.0.0\"
        [[bin]]
        name = \"bin\"
        [dependencies]
        {}
    ",
                       script_name.replace('.', "_"),
                       crates_to_toml_format(extern_crates));
    let mut toml_f =
        fs::File::create(project_dir.join("Cargo.toml")).chain_err(|| "Unable to create Cargo.toml")?;
    toml_f.write_all(toml.as_bytes()).chain_err(|| "Unable to write to Cargo.toml")?;
    Ok(())
}

fn format_code(extern_crates: &[CrateImport], code: String) -> String {
    format!("{}
fn main() {{
    {}
}}\n",
            crates_to_code(extern_crates),
            indent(remove_shebang(&code).trim()))
}

fn remove_shebang(code: &str) -> &str {
    let start = if code.starts_with("#!") {
        code.find('\n').unwrap_or(code.len())
    } else {
        0
    };
    &code[start..]
}

fn indent(code: &str) -> String {
    code.replace('\n', "\n    ")
}

fn extract_extern_crates(code: String) -> Result<(Vec<CrateImport>, String)> {
    let macro_regex = r"(?P<macro_use>#\[macro_use\])?";
    let crate_name_regex = r"(?P<name>[^; \[]+)";
    let crate_version_regex = r"(?:\[(?P<version>[^\]]+)\])?";
    let regex = Regex::new(&format!(r"{}\s*extern\s+crate\s+{}{};",
                                    macro_regex,
                                    crate_name_regex,
                                    crate_version_regex))
        .unwrap();
    let mut crates = Vec::new();
    let mut new_code = String::new();
    let mut last_start = 0;
    for capture in regex.captures_iter(&code) {
        let macro_use = capture.name("macro_use").is_some();
        let name = capture.name("name").unwrap();
        let version = capture.name("version").unwrap_or("*");
        let crate_ = CrateImport::new(name, version, macro_use);
        println!("found crate: {:?}", &crate_);
        crates.push(crate_);

        let (start, end) = capture.pos(0).unwrap();
        new_code.push_str(&code[last_start..start]);
        last_start = end;
    }
    new_code.push_str(&code[last_start..]);
    Ok((crates, new_code))
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let mut f = fs::File::open(path).chain_err(|| "Unable to read file")?;
    let mut content = String::new();
    f.read_to_string(&mut content).chain_err(|| "Unable to read file")?;
    Ok(content)
}

fn compile<P: AsRef<Path>>(project_dir: P) -> Result<()> {
    let mut child = Command::new("cargo").arg("build")
        .arg("--release")
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .chain_err(|| "Unable to start the compiler")?;
    let rustc_result = child.wait().chain_err(|| "cargo crashed")?;
    if !rustc_result.success() {
        io::copy(&mut child.stdout.unwrap(), &mut io::stdout())
            .chain_err(|| "Unable to write compiler stdout")?;
        io::copy(&mut child.stderr.unwrap(), &mut io::stderr())
            .chain_err(|| "Unable to write compiler stderr")?;
        bail!("The script did not compile successfully")
    } else {
        Ok(())
    }
}

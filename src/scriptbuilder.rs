use std::path::Path;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use {Result, ResultExt};

use syntex_syntax::parse::{self, ParseSess};
use syntex_syntax::ast::{ItemKind, Crate};

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
    fs::copy(script_path, &main_path).chain_err(|| "Unable to copy script to main.rs")?;

    create_toml(&script_name, &output_crate_dir)?;
    compile(output_crate_dir)
}

fn parse_crate<P: AsRef<Path>>(path: P) -> Result<Crate> {
    let parse_session = ParseSess::new();
    let result = parse::parse_crate_from_file(path.as_ref(), &parse_session);
    result.map_err(|mut e| {
            e.cancel();
            io::Error::new(io::ErrorKind::Other,
                           e.into_diagnostic().message().to_owned())
        })
        .chain_err(|| "Unable to parse script")
}

fn list_extern_crates(crate_: &Crate) -> Result<Vec<String>> {
    let mut crates = Vec::new();
    for item in &crate_.module.items {
        if let ItemKind::ExternCrate(..) = item.node {
            crates.push(format!("{}", item.ident.name.as_str()));
        }
    }
    Ok(crates)
}

fn create_toml<P: AsRef<Path>>(script_name: &str, project_dir: P) -> Result<()> {
    let project_dir = project_dir.as_ref();
    let main_path = project_dir.join("src").join("main.rs");
    let parsed_crate = parse_crate(main_path)?;
    let extern_crates = list_extern_crates(&parsed_crate)?;
    let toml = format!("[package]
        name = \"{}\"
        version = \"0.0.0\"
        [[bin]]
        name = \"bin\"
        [dependencies]
        {}
    ",
                       script_name.replace('.', "_"),
                       extern_crates_to_toml_format(&extern_crates));
    let mut toml_f =
        fs::File::create(project_dir.join("Cargo.toml")).chain_err(|| "Unable to create Cargo.toml")?;
    toml_f.write_all(toml.as_bytes()).chain_err(|| "Unable to write to Cargo.toml")?;
    Ok(())
}

fn extern_crates_to_toml_format(crates: &[String]) -> String {
    crates.into_iter().fold(String::new(), |mut acc, crate_name| {
        acc.push_str(&crate_name);
        acc.push_str(" = \"*\"\n");
        acc
    })
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

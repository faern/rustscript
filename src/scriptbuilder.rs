use std::path::Path;
use std::fs;
use std::io::{self, Write, BufReader, BufRead};
use std::process::{Command, Stdio};

use {Result, ResultExt};

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

fn create_toml<P: AsRef<Path>>(script_name: &str, project_dir: P) -> Result<()> {
    let project_dir = project_dir.as_ref();
    let main_path = project_dir.join("src").join("main.rs");
    let extern_crates = list_extern_crates(&main_path)?;
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

fn list_extern_crates<P: AsRef<Path>>(script_path: P) -> Result<Vec<String>> {
    let script_path = script_path.as_ref();
    let f = fs::File::open(script_path).chain_err(|| "Unable to read script")?;
    let buf_f = BufReader::new(f);
    let mut crates = Vec::new();
    for line in buf_f.lines() {
        let line = line.chain_err(|| "Unable to read script")?;
        let parts: Vec<String> = line.trim().split_whitespace().map(|s| s.to_owned()).collect();
        if parts.len() == 3 && ["extern", "crate"] == parts[..2] && parts[2].ends_with(";") {
            let crate_name = parts[2].trim_right_matches(';').to_owned();
            crates.push(crate_name);
        }
    }
    Ok(crates)
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

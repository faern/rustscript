// #![feature(rustc_private)]
#![recursion_limit = "1024"]

extern crate app_dirs;
extern crate crypto;
#[macro_use]
extern crate error_chain;

extern crate syntex_syntax;
extern crate syntex_errors;

mod errors {
    error_chain!{}
}

use errors::*;

mod cache;
mod hash;

use syntex_syntax::parse::{self, ParseSess};
use syntex_syntax::ast::ItemKind;


use std::env::args;
use std::process::{self, Command, ExitStatus};
use std::path::Path;
use std::fs;
use std::io::{self, Write};

const SCRIPT_BIN_PATH: &'static str = "target/release/bin";

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = args();
    let _bin_path = args.next().ok_or("rustscript not given as first argument")?;
    let script_path_str = args.next().ok_or("No script given to run")?;
    let script_path = Path::new(&script_path_str);
    let script_args: Vec<String> = args.collect();

    let hash = hash::hash(&script_path).chain_err(|| "Unable to take signature of script")?;

    let cache = cache::BinCache::new()?;
    let script_cache_dir = cache.get(hash)?;
    let script_bin_path = script_cache_dir.join(SCRIPT_BIN_PATH);
    if !script_bin_path.exists() {
        println!("Script not found in cache, compiling");
        build_script_crate(&script_path, &script_cache_dir).chain_err(|| "Unable to compile the script")?;
    }

    println!("Running {} with {:?}", &script_path_str, script_args);

    let mut script_process = Command::new(&script_bin_path).args(&script_args)
        .spawn()
        .chain_err(|| "Unable to spawn script")?;
    let script_result = script_process.wait().chain_err(|| "Script crashed")?;
    terminate_like_script(script_result);
}

#[cfg(not(unix))]
fn terminate_like_script(exit_status: ExitStatus) -> ! {
    process::exit(exit_status.code().unwrap());
}

#[cfg(unix)]
fn terminate_like_script(exit_status: ExitStatus) -> ! {
    use std::os::unix::process::ExitStatusExt;
    process::exit(match exit_status.code() {
        Some(i) => i,
        None => exit_status.signal().unwrap(),
    });
}

fn build_script_crate<P: AsRef<Path>, Q: AsRef<Path>>(script_path: P, cache_dir: Q) -> Result<()> {
    list_extern_crates(&script_path);

    let script_path = script_path.as_ref();
    let script_name = script_path.file_name()
        .ok_or("Script has no name")?
        .to_str()
        .ok_or("Script name is not valid utf-8")?;
    let cache_dir = cache_dir.as_ref();
    let src_dir = cache_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir).chain_err(|| "unable to create src dir")?;
    }
    let main_path = src_dir.join("main.rs");
    fs::copy(script_path, &main_path).chain_err(|| "Unable to copy script to main.rs")?;

    create_toml(&script_name, &cache_dir)?;
    compile(cache_dir)
}

fn list_extern_crates<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let parse_session = ParseSess::new();
    let result =
        parse::parse_crate_from_file(path.as_ref(), &parse_session).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))
            .chain_err(|| "Unable to parse script")?;
    let mut crates = Vec::new();
    for item in result.module.items {
        if let ItemKind::ExternCrate(..) = item.node {
            println!("CRATE: {}", item.ident.name.as_str());
            crates.push(format!("{}", item.ident.name.as_str()));
        }
    }
    Ok(crates)
}

fn create_toml<P: AsRef<Path>>(script_name: &str, project_dir: P) -> Result<()> {
    let project_dir = project_dir.as_ref();
    let main_path = project_dir.join("src").join("main.rs");
    let extern_crates = list_extern_crates(main_path)?;
    let toml = format!("[package]
        name = \"{}\"
        version = \"0.0.0\"
        [[bin]]
        name = \"bin\"
        [dependencies]
        {}
    ",
                       script_name.replace('.', "_"),
                       extern_crates.into_iter().fold(String::new(), |mut acc, crate_name| {
                           acc.push_str(&crate_name);
                           acc.push_str(" = \"*\"\n");
                           acc
                       }));
    let mut toml_f =
        fs::File::create(project_dir.join("Cargo.toml")).chain_err(|| "Unable to create Cargo.toml")?;
    toml_f.write_all(toml.as_bytes()).chain_err(|| "Unable to write to Cargo.toml")?;
    Ok(())
}

fn compile<P: AsRef<Path>>(project_dir: P) -> Result<()> {
    let mut rustc = Command::new("cargo").arg("build")
        .arg("--release")
        .current_dir(project_dir)
        .spawn()
        .chain_err(|| "Unable to start the compiler")?;
    let rustc_result = rustc.wait().chain_err(|| "cargo crashed")?;
    if !rustc_result.success() {
        bail!("rustc did not build successfully")
    } else {
        Ok(())
    }
}

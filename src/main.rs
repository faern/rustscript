#![recursion_limit = "1024"]

extern crate app_dirs;
extern crate crypto;
#[macro_use]
extern crate error_chain;

mod errors {
    error_chain!{}
}

use errors::*;

mod cache;
mod hash;

use std::env::args;
use std::process::{self, Command, ExitStatus};
use std::path::Path;
use std::fs;
use std::io::Write;

const SCRIPT_BIN_PATH: &'static str = "target/release/rustscript";

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
    let script_path = args.next().ok_or("No script given to run")?;
    let script_args: Vec<String> = args.collect();

    let hash = hash::hash(&script_path).chain_err(|| "Unable to take signature of script")?;

    let cache = cache::BinCache::new()?;
    let script_cache_dir = cache.get(hash)?;
    let script_bin_path = script_cache_dir.join(SCRIPT_BIN_PATH);
    if !script_bin_path.exists() {
        println!("Script not found in cache, compiling");
        compile(&script_path, &script_cache_dir).chain_err(|| "Unable to compile the script")?;
    }

    println!("Running {} with {:?}", &script_path, script_args);

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

fn compile<P: AsRef<Path>, Q: AsRef<Path>>(script_path: P, cache_dir: Q) -> Result<()> {
    let script_path = script_path.as_ref();
    let cache_dir = cache_dir.as_ref();
    let src_dir = cache_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir).chain_err(|| "unable to create src dir")?;
    }
    let main_path = src_dir.join("main.rs");
    fs::copy(script_path, &main_path).chain_err(|| "Unable to copy script to main.rs")?;

    let toml = "[package]
        name = \"rustscript\"
        version = \"0.0.0\"
        ";
    let mut toml_f =
        fs::File::create(cache_dir.join("Cargo.toml")).chain_err(|| "Unable to create Cargo.toml")?;
    toml_f.write_all(toml.as_bytes()).chain_err(|| "Unable to write to Cargo.toml")?;

    let mut rustc = Command::new("cargo").arg("build")
        .arg("--release")
        .current_dir(cache_dir)
        .spawn()
        .chain_err(|| "Unable to start the compiler")?;
    let rustc_result = rustc.wait().chain_err(|| "cargo crashed")?;
    if !rustc_result.success() {
        bail!("rustc did not build successfully")
    } else {
        Ok(())
    }
}

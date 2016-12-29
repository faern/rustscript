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
use std::process::{self, Command};
use std::path::Path;

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
    let script_bin_path = cache.get(hash);
    if !script_bin_path.exists() {
        println!("Script not found in cache, compiling");
        compile(&script_path, &script_bin_path).chain_err(|| "Unable to compile the script")?;
    }

    println!("Running {} with {:?}", &script_path, script_args);

    let mut script_process = Command::new(&script_bin_path).args(&script_args)
        .spawn()
        .chain_err(|| "Unable to spawn script")?;
    let script_result = script_process.wait().chain_err(|| "Script crashed")?;
    process::exit(script_result.code().unwrap());
}

fn compile<P: AsRef<Path>, P2: AsRef<Path>>(script_path: P, output_path: P2) -> Result<()> {
    let mut rustc = Command::new("rustc").arg(script_path.as_ref())
        .arg("-o")
        .arg(output_path.as_ref())
        .spawn()
        .chain_err(|| "Unable to spawn rustc")?;
    let rustc_result = rustc.wait().chain_err(|| "rustc crashed")?;
    if !rustc_result.success() {
        bail!("rustc did not build successfully")
    } else {
        Ok(())
    }
}

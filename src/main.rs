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
mod runner;
mod scriptbuilder;

use std::env::args;
use std::path::Path;

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
        scriptbuilder::build_script_crate(&script_path, &script_cache_dir).chain_err(|| "Unable to compile the script")?;
    }
    runner::run_script(&script_bin_path, script_args)
}

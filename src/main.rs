// #![feature(rustc_private)]
#![recursion_limit = "1024"]

extern crate app_dirs;
extern crate crypto;
#[macro_use]
extern crate error_chain;
extern crate regex;
extern crate semver;
#[macro_use]
extern crate clap;

mod errors {
    error_chain!{}
}
use errors::*;

mod cache;
mod hash;
mod runner;
mod scriptbuilder;

use clap::{App, Arg, AppSettings};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

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
    let app = create_app();
    let matches = app.get_matches();
    let verbosity_level = matches.occurrences_of("verbose");
    let recompile = matches.is_present("recompile");
    let no_run = matches.is_present("no_run");
    let script_path = matches.value_of("script_path").unwrap();
    let script_args = values_t!(matches, "script_args", String).unwrap();

    let (path_hash, script_hash) =
        hash::hash_script(&script_path).chain_err(|| "Unable to take signature of script")?;

    let cache = cache::BinCache::new()?;
    let script_cache_dir = cache.get(path_hash)?;
    let script_hash_path = script_cache_dir.join("script_hash");
    let cached_script_hash = file_to_string(&script_hash_path).unwrap_or(String::new());
    let script_bin_path = script_cache_dir.join(SCRIPT_BIN_PATH);
    if !script_bin_path.exists() || recompile || script_hash != cached_script_hash {
        scriptbuilder::build_script_crate(&script_path, &script_cache_dir, verbosity_level > 0).chain_err(|| "Unable to compile the script")?;
        string_to_file(&script_hash_path, &script_hash)?
    }
    if !no_run {
        runner::run_script(&script_bin_path, script_args)?;
    }
    Ok(())
}

fn file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let mut f = File::open(path).chain_err(|| format!("Unable to open {:?}", path))?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).chain_err(|| "Unable to read script")?;
    Ok(buffer)
}

fn string_to_file<P: AsRef<Path>>(path: P, data: &str) -> Result<()> {
    let path = path.as_ref();
    let mut f = File::create(path).chain_err(|| format!("Unable to create {:?}", path))?;
    f.write_all(data.as_bytes()).chain_err(|| format!("Unable to write {:?}", path))
}

fn create_app() -> App<'static, 'static> {
    let verbose_arg = Arg::with_name("verbose")
        .short("v")
        .multiple(true)
        .help("Print the output of the script compilation");
    let recompile_arg = Arg::with_name("recompile")
        .short("r")
        .long("recompile")
        .help("Force the script to be compiled, even if it is already in the cache");
    let no_run_arg = Arg::with_name("no_run")
        .long("no-run")
        .help("Don't run the script, only compile it. Add --recompile to compile even if it is \
               in the cache.");
    let script_path_arg = Arg::with_name("script_path").index(1).required(true);
    let script_args_arg = Arg::with_name("script_args").index(2).multiple(true).default_value("");
    App::new("rustscript")
        .author(crate_authors!())
        .version(crate_version!())
        .settings(&[AppSettings::TrailingVarArg])
        .arg(verbose_arg)
        .arg(recompile_arg)
        .arg(no_run_arg)
        .arg(script_path_arg)
        .arg(script_args_arg)
}

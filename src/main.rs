extern crate app_dirs;
extern crate crypto;

mod cache;
mod hash;

use std::env::args;
use std::process::{self, Command};
use std::path::Path;

fn main() {
    let mut args = args();
    let _bin_path = args.next().expect("wtf?");
    let script_path = args.next().expect("No script given");
    let script_args: Vec<String> = args.collect();

    let hash = hash::hash(&script_path).unwrap();

    let cache = cache::BinCache::new();
    let script_bin_path = cache.get(hash);
    if !script_bin_path.exists() {
        if let Err(e) = compile(&script_path, &script_bin_path) {
            println!("Unable to compile the script: {}", e);
            process::exit(1);
        }
    }

    println!("Running {} with {:?}", &script_path, script_args);

    let mut script_process =
        Command::new("./a.out").args(&script_args).spawn().expect("Unable to spawn script");
    script_process.wait().expect("Script crashed");
}

fn compile<P: AsRef<Path>, P2: AsRef<Path>>(script_path: P, output_path: P2) -> Result<(), String> {
    let mut rustc = Command::new("rustc").arg(script_path.as_ref())
        .arg("-o")
        .arg(output_path.as_ref())
        .spawn()
        .map_err(|_| "Unable to spawn rustc")?;
    let rustc_result = rustc.wait().map_err(|_| "Rustc crashed")?;
    if !rustc_result.success() {
        Err("Rustc did not build successfully".to_owned())
    } else {
        Ok(())
    }
}

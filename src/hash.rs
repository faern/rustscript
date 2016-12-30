use crypto::digest::Digest;
use crypto::sha2::Sha256;

use {Result, ResultExt};

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn hash_script<P: AsRef<Path>>(script_path: P) -> Result<(String, String)> {
    let mut f = File::open(&script_path).chain_err(|| "Unable to open script file")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).chain_err(|| "Unable to read script")?;
    Ok((hash_path(script_path)?, hash(&buffer)))
}

fn hash_path<P: AsRef<Path>>(p: P) -> Result<String> {
    // let current_dir = ::std::env::current_dir().error_chain(|| "Unable to get current directory")?;
    let absolute_path =
        p.as_ref().canonicalize().chain_err(|| "Unable to get absolute path to script")?;
    Ok(hash(absolute_path.to_str().unwrap().as_bytes()))
}

fn hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(data);
    hasher.result_str()
}

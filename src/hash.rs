use crypto::digest::Digest;
use crypto::sha2::Sha256;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn hash<P: AsRef<Path>>(script_path: P) -> io::Result<String> {
    let mut f = File::open(script_path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.input(&buffer);
    Ok(hasher.result_str())
}

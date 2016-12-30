use app_dirs::{app_root, AppInfo, AppDataType};

use std::path::PathBuf;
use std::fs;

use {ResultExt, Result};

const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_NAME"),
};

pub struct BinCache {
    cache_dir: PathBuf,
}

impl BinCache {
    pub fn new() -> Result<Self> {
        let cache_dir = app_root(AppDataType::UserCache, &APP_INFO).chain_err(|| "No cache dir")?;
        Ok(BinCache { cache_dir: cache_dir })
    }

    pub fn get(&self, hash: String) -> Result<PathBuf> {
        let mut path = self.cache_dir.clone();
        path.push(hash);
        if !path.exists() {
            fs::create_dir(&path).chain_err(|| "Unable to create script cache dir")?;
        }
        Ok(path)
    }
}

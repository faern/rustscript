use app_dirs::{app_root, AppInfo, AppDataType};
use std::path::PathBuf;

use {ResultExt, Result};

const APP_INFO: AppInfo = AppInfo {
    name: "rustscript",
    author: "33c3",
};

pub struct BinCache {
    cache_dir: PathBuf,
}

impl BinCache {
    pub fn new() -> Result<Self> {
        let cache_dir = app_root(AppDataType::UserCache, &APP_INFO).chain_err(|| "No cache dir")?;
        Ok(BinCache { cache_dir: cache_dir })
    }

    pub fn get(&self, hash: String) -> PathBuf {
        let mut path = self.cache_dir.clone();
        path.push(hash);
        path
    }
}

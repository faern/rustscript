use app_dirs::{app_root, AppInfo, AppDataType};
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo {
    name: "rustscript",
    author: "33c3",
};

pub struct BinCache {
    cache_dir: PathBuf,
}

impl BinCache {
    pub fn new() -> Self {
        BinCache { cache_dir: app_root(AppDataType::UserCache, &APP_INFO).expect("No cache dir") }
    }

    pub fn get(&self, hash: String) -> PathBuf {
        let mut path = self.cache_dir.clone();
        path.push(hash);
        path
    }
}

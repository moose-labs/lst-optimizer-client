use std::path::PathBuf;

use lst_optimizer_utils::path::{get_workspace_file, resolve_path};

pub fn get_package_file(f: &str) -> PathBuf {
    resolve_path("packages/lst-optimizer-client").join(f)
}

pub fn get_registry_file() -> PathBuf {
    get_workspace_file("registry.toml")
}

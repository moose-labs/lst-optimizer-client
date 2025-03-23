use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

pub fn get_workspace_root() -> PathBuf {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch workspace metadata");

    metadata.workspace_root.into_std_path_buf()
}

pub fn resolve_path(path: &str) -> PathBuf {
    get_workspace_root().join(path)
}

pub fn get_workspace_file(f: &str) -> PathBuf {
    resolve_path("./").join(f)
}

pub fn get_deps_configs(f: &str) -> PathBuf {
    resolve_path("deps/configs").join(f)
}

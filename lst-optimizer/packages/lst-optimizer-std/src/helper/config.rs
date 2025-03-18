use anyhow::Result;
use serde::Deserialize;

use crate::types::{asset::Asset, asset_repository::AssetRepository};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub lst_list: Vec<Asset>,
}

pub fn asset_repository_from_toml(path: &str) -> Result<AssetRepository> {
    let assets_toml = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&assets_toml)?;
    Ok(AssetRepository::new(config.lst_list))
}

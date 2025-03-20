use anyhow::Result;

use super::asset::Asset;

#[derive(Debug, Clone)]
pub struct AssetRepository {
    assets: Vec<Asset>,
}

impl AssetRepository {
    pub fn new(assets: Vec<Asset>) -> Self {
        Self { assets }
    }

    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.clone()
    }

    pub fn get_asset_from_mint(&self, mint: &str) -> Result<Asset> {
        for asset in self.assets.iter() {
            if asset.mint.eq(mint) {
                return Ok(asset.clone());
            }
        }
        Err(anyhow::anyhow!(
            "Asset mint {} not found in the repository",
            mint
        ))
    }

    pub fn get_asset_from_symbol(&self, symbol: &str) -> Result<Asset> {
        let symbol = symbol.to_lowercase();
        for asset in self.assets.iter() {
            if asset.symbol.to_lowercase().eq(&symbol) {
                return Ok(asset.clone());
            }
        }
        Err(anyhow::anyhow!(
            "Asset symbol {} not found in the repository",
            symbol
        ))
    }
}

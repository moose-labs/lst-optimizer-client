use anyhow::Result;

use super::asset::Asset;

#[derive(Debug, Clone)]
pub struct AssetRepository {
    assets: Vec<Asset>,
}

impl AssetRepository {
    pub fn new(assets: Vec<Asset>) -> Self {
        Self {
            assets,
        }
    }

    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.clone()
    }

    pub fn get_asset(&self, mint: &str) -> Result<Asset> {
        for asset in self.assets.iter() {
            if asset.mint.eq(mint) {
                return Ok(asset.clone());
            }
        }
        Err(anyhow::anyhow!("Asset {} not found in the repository", mint))
    }
}

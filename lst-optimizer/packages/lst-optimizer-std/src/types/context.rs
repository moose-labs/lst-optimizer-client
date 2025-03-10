use super::asset_repository::AssetRepository;
use anyhow::Result;
use super::asset::Asset;

#[derive(Debug, Clone)]
pub struct Context {
    pub payer: String,
    pub asset_repository: AssetRepository,
}

impl Context {
    pub fn with_asset_repository(self, asset_repository: AssetRepository) -> Self {
        Self {
            asset_repository,
            ..self
        }
    }

    pub fn with_payer(self, payer: String) -> Self {
        Self {
            payer,
            ..self
        }
    }

    pub fn get_asset(&self, mint: &str) -> Result<Asset> {
        self.asset_repository.get_asset(mint)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            asset_repository: AssetRepository::new(vec![]),
            payer: "".to_string(),
        }
    }
}

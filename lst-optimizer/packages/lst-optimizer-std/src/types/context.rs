use super::asset::Asset;
use super::asset_repository::AssetRepository;
use anyhow::Result;
use log::info;

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
        info!("Setting payer to {}", payer);
        Self { payer, ..self }
    }

    pub fn get_known_asset_from_mint(&self, mint: &str) -> Result<Asset> {
        self.asset_repository.get_asset_from_mint(mint)
    }

    pub fn get_known_asset_from_symbol(&self, symbol: &str) -> Result<Asset> {
        self.asset_repository.get_asset_from_symbol(symbol)
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

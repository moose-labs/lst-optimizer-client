use super::asset::Asset;
use super::asset_repository::AssetRepository;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

#[derive(Debug)]
pub struct Context {
    payer: Keypair,
    asset_repository: AssetRepository,
}

impl Context {
    pub fn with_asset_repository(self, asset_repository: AssetRepository) -> Self {
        Self {
            asset_repository,
            ..self
        }
    }

    pub fn with_payer(self, payer: Keypair) -> Self {
        Self { payer, ..self }
    }

    pub fn get_known_asset_from_mint(&self, mint: &str) -> Result<Asset> {
        self.asset_repository.get_asset_from_mint(mint)
    }

    pub fn get_known_asset_from_symbol(&self, symbol: &str) -> Result<Asset> {
        self.asset_repository.get_asset_from_symbol(symbol)
    }

    pub fn get_kwown_assets(&self) -> Vec<Asset> {
        self.asset_repository.get_assets()
    }

    pub fn get_payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn get_payer(&self) -> &Keypair {
        &self.payer
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            asset_repository: AssetRepository::new(vec![]),
            payer: Keypair::new(),
        }
    }
}

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::controller::ControllerClient;

// Mint state helper

#[async_trait::async_trait]
pub trait MintAccountQuery {
    async fn mint(&self, pubkey: &Pubkey) -> Result<spl_token_2022::state::Mint>;
    async fn owner(&self, pubkey: &Pubkey) -> Result<Pubkey>;
}

#[async_trait::async_trait]
impl MintAccountQuery for ControllerClient {
    async fn mint(&self, pubkey: &Pubkey) -> Result<spl_token_2022::state::Mint> {
        let rpc = self.rpc_client();
        let lp_mint_acc = rpc.get_account(pubkey).await?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
                &lp_mint_acc.data,
            )?;
        Ok(state.base)
    }

    async fn owner(&self, pubkey: &Pubkey) -> Result<Pubkey> {
        let rpc = self.rpc_client();
        let lp_mint_acc = rpc.get_account(pubkey).await?;
        Ok(lp_mint_acc.owner)
    }
}

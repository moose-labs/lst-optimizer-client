use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[async_trait::async_trait]
pub trait MintAccountQuery {
    async fn get_mint(&self, rpc: &RpcClient) -> Result<spl_token_2022::state::Mint>;
    async fn get_mint_owner(&self, rpc: &RpcClient) -> Result<Pubkey>;
}

#[async_trait::async_trait]
impl MintAccountQuery for Pubkey {
    async fn get_mint(&self, rpc: &RpcClient) -> Result<spl_token_2022::state::Mint> {
        let lp_mint_acc = rpc.get_account(self).await?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
                &lp_mint_acc.data,
            )?;
        Ok(state.base)
    }

    async fn get_mint_owner(&self, rpc: &RpcClient) -> Result<Pubkey> {
        let lp_mint_acc = rpc.get_account(self).await?;
        Ok(lp_mint_acc.owner)
    }
}

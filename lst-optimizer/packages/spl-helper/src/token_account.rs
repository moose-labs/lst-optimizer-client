use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::mint::MintAccountQuery;

#[async_trait::async_trait]
pub trait TokenAccountQuery {
    async fn get_token_account_balance(&self, rpc: &RpcClient) -> Result<u64>;
    async fn get_associated_token_account_with_program_id(
        &self,
        wallet_address: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<Pubkey>;
    async fn resolve_associated_token_account(
        &self,
        wallet_address: &Pubkey,
        rpc: &RpcClient,
    ) -> Result<Pubkey>;
}

#[async_trait::async_trait]
impl TokenAccountQuery for Pubkey {
    async fn get_token_account_balance(&self, rpc: &RpcClient) -> Result<u64> {
        let acc = rpc.get_account(self).await?;
        let token_account = spl_token_2022::state::Account::unpack(&acc.data)?;
        Ok(token_account.amount)
    }

    async fn get_associated_token_account_with_program_id(
        &self,
        wallet_address: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<Pubkey> {
        let pk =
            get_associated_token_address_with_program_id(wallet_address, &self, token_program_id);
        Ok(pk)
    }

    async fn resolve_associated_token_account(
        &self,
        wallet_address: &Pubkey,
        rpc: &RpcClient,
    ) -> Result<Pubkey> {
        let program_id = self.get_mint_owner(rpc).await?;
        let pk = self
            .get_associated_token_account_with_program_id(wallet_address, &program_id)
            .await?;
        Ok(pk)
    }
}

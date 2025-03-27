pub mod jitosol;
pub mod msol;
pub mod wsol;

use moose_utils::result::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

#[async_trait::async_trait]
pub trait LstKeys {
    fn get_lsl_mint() -> Pubkey;
    fn get_calculator_program_id() -> Pubkey;
    fn get_token_program_id() -> Pubkey;
    async fn fetch_account_metas(rpc: &RpcClient) -> Result<Vec<AccountMeta>>;
}

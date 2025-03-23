pub mod jitosol;
pub mod msol;
pub mod wsol;

use moose_utils::result::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

#[async_trait::async_trait]
pub trait LstKeys {
    fn get_cal_program_and_mint() -> (Pubkey, Pubkey);
    fn get_token_program_id() -> Pubkey;
    async fn fetch_account_metas(rpc: &RpcClient) -> Result<Vec<AccountMeta>>;
}

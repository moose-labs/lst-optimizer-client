use controller_lib::calculator::typedefs::CalculatorType;
use moose_utils::result::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use super::LstKeys;

pub struct WsolKeys {}

#[async_trait::async_trait]
impl LstKeys for WsolKeys {
    fn get_lsl_mint() -> Pubkey {
        wsol_keys::wsol::ID
    }

    fn get_calculator_program_id() -> Pubkey {
        wsol_calculator_lib::program::ID
    }

    fn get_token_program_id() -> Pubkey {
        spl_token::id()
    }

    async fn fetch_account_metas(rpc: &RpcClient) -> Result<Vec<AccountMeta>> {
        Ok(CalculatorType::Wsol.fetch_account_metas(rpc).await?)
    }
}

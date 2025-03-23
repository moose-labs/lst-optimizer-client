use controller_lib::calculator::typedefs::CalculatorType;
use moose_utils::result::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use super::LstKeys;

pub struct MarinadeKeys {}

#[async_trait::async_trait]
impl LstKeys for MarinadeKeys {
    fn get_cal_program_and_mint() -> (Pubkey, Pubkey) {
        (
            marinade_calculator_lib::program::ID,
            marinade_keys::msol::ID,
        )
    }

    fn get_token_program_id() -> Pubkey {
        spl_token::id()
    }

    async fn fetch_account_metas(rpc: &RpcClient) -> Result<Vec<AccountMeta>> {
        Ok(CalculatorType::Marinade.fetch_account_metas(rpc).await?)
    }
}

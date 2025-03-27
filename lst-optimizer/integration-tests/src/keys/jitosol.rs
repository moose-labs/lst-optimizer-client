use controller_lib::calculator::typedefs::CalculatorType;
use moose_utils::result::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use super::LstKeys;

pub struct JitoKeys {}

#[async_trait::async_trait]
impl LstKeys for JitoKeys {
    fn get_lsl_mint() -> Pubkey {
        pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn")
    }

    fn get_calculator_program_id() -> Pubkey {
        spl_calculator_lib::program::ID
    }

    fn get_token_program_id() -> Pubkey {
        spl_token::id()
    }

    async fn fetch_account_metas(rpc: &RpcClient) -> Result<Vec<AccountMeta>> {
        Ok(
            CalculatorType::Spl("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb".to_string())
                .fetch_account_metas(rpc)
                .await?,
        )
    }
}

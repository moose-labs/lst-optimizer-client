use lido_calculator_lib::lido_sol_val_calc_account_metas;
use marinade_calculator_lib::marinade_sol_val_calc_account_metas;
use sanctum_token_ratio::U64ValueRange;

use anyhow::Result;
use solana_readonly_account::keyed::Keyed;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};
use spl_calculator_lib::SplLstSolCommonFreeArgsConst;
use wsol_calculator_lib::WSOL_LST_SOL_COMMON_METAS;

use crate::{controller::ControllerClient, controller_instructions::ControllerInstructionBuilder};

use super::{
    instructions::CalculatorInstructions,
    typedefs::{parse_u64_value_range_return_data, CalculatorType},
};

// Convert between LST and SOL using the calculator program

#[async_trait::async_trait]
pub trait CalculatorQuery {
    async fn convert_sol_to_lst(
        &self,
        payer: &Pubkey,
        calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<U64ValueRange>;

    async fn convert_lst_to_sol(
        &self,
        payer: &Pubkey,
        calculator_type: CalculatorType,
        amount: u64,
    ) -> Result<U64ValueRange>;

    async fn fetch_calculator_account_metas(
        &self,
        calculator_type: &CalculatorType,
    ) -> Result<Vec<AccountMeta>>;
}

#[async_trait::async_trait]
impl CalculatorQuery for ControllerClient {
    async fn convert_sol_to_lst(
        &self,
        payer: &Pubkey,
        calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<U64ValueRange> {
        let builder = ControllerInstructionBuilder::new();
        let accounts = self
            .fetch_calculator_account_metas(&calculator_type)
            .await?;
        let instruction =
            builder.create_sol_to_lst_instruction(calculator_type, lamports, accounts)?;
        let return_data = self.simulate_instructions(payer, &[instruction]).await?;
        let val = parse_u64_value_range_return_data(&return_data)?;
        Ok(val)
    }

    async fn convert_lst_to_sol(
        &self,
        payer: &Pubkey,
        calculator_type: CalculatorType,
        amount: u64,
    ) -> Result<U64ValueRange> {
        let builder = ControllerInstructionBuilder::new();
        let accounts = self
            .fetch_calculator_account_metas(&calculator_type)
            .await?;
        let instruction =
            builder.create_lst_to_sol_instruction(calculator_type, amount, accounts)?;
        let return_data = self.simulate_instructions(payer, &[instruction]).await?;
        let val = parse_u64_value_range_return_data(&return_data)?;
        Ok(val)
    }

    async fn fetch_calculator_account_metas(
        &self,
        calculator_type: &CalculatorType,
    ) -> Result<Vec<AccountMeta>> {
        let rpc = self.rpc_client();
        let accs = match calculator_type {
            CalculatorType::Lido => lido_sol_val_calc_account_metas().to_vec(),
            CalculatorType::Marinade => marinade_sol_val_calc_account_metas().to_vec(),
            CalculatorType::Wsol => WSOL_LST_SOL_COMMON_METAS.to_vec(),
            CalculatorType::Spl(pool)
            | CalculatorType::SanctumSpl(pool)
            | CalculatorType::SanctumSplMulti(pool) => {
                let pool: Pubkey = pool.parse()?;
                let pool_acc = rpc.get_account(&pool).await?;
                let reso = SplLstSolCommonFreeArgsConst {
                    spl_stake_pool: Keyed {
                        account: pool_acc,
                        pubkey: pool,
                    },
                };
                (match calculator_type {
                    CalculatorType::Spl(_) => reso.resolve_spl_to_account_metas(),
                    CalculatorType::SanctumSpl(_) => reso.resolve_sanctum_spl_to_account_metas(),
                    CalculatorType::SanctumSplMulti(_) => {
                        reso.resolve_sanctum_spl_multi_to_account_metas()
                    }
                    _ => unreachable!(),
                })?
                .to_vec()
            }
        };

        Ok(accs)
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey;

    use super::*;

    #[tokio::test]
    async fn test_calculators() {
        let payer = &pubkey!("86naSVEnAUH1C9b4WktPqohydNhW5c1Tnt2foQqnZKb1");
        let url = "https://mainnet.helius-rpc.com/?api-key=f48894c7-d3cd-406a-8bec-bc29c3f9052e"
            .to_string();
        let rpc = solana_client::nonblocking::rpc_client::RpcClient::new(url);
        let client = ControllerClient::new(rpc);
        let amount_sol = client
            .convert_lst_to_sol(payer, CalculatorType::Marinade, 1_000_000_000)
            .await
            .unwrap();
        assert_ne!(amount_sol.clone().get_min(), 0);
        println!("1 mSOL = {:?} SOL", amount_sol.get_min());

        let amount_lst = client
            .convert_sol_to_lst(payer, CalculatorType::Marinade, 1_000_000_000)
            .await
            .unwrap();
        assert_ne!(amount_sol.clone().get_min(), 0);
        println!("1 SOL = {:?} mSOL", amount_lst.get_min());
    }
}

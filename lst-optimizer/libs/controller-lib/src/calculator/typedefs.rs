use anyhow::Result;
use lido_calculator_lib::lido_sol_val_calc_account_metas;
use marinade_calculator_lib::marinade_sol_val_calc_account_metas;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_readonly_account::keyed::Keyed;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};
use spl_calculator_lib::SplLstSolCommonFreeArgsConst;
use wsol_calculator_lib::WSOL_LST_SOL_COMMON_METAS;

#[derive(Clone, Debug)]
pub enum CalculatorType {
    Lido,
    Marinade,
    Wsol,
    Spl(String),
    SanctumSpl(String),
    SanctumSplMulti(String),
}

// Metadata for the common accounts used by all calculators

pub fn calculator_program_id(calculator_type: &CalculatorType) -> Pubkey {
    match calculator_type {
        CalculatorType::Lido => lido_calculator_lib::program::ID,
        CalculatorType::Marinade => marinade_calculator_lib::program::ID,
        CalculatorType::Wsol => wsol_calculator_lib::program::ID,
        CalculatorType::Spl(_) => spl_calculator_lib::program::ID,
        CalculatorType::SanctumSpl(_) => spl_calculator_lib::sanctum_spl_sol_val_calc_program::ID,
        CalculatorType::SanctumSplMulti(_) => {
            spl_calculator_lib::sanctum_spl_multi_sol_val_calc_program::ID
        }
    }
}

impl CalculatorType {
    pub async fn fetch_account_metas(
        self: &CalculatorType,
        rpc: &RpcClient,
    ) -> Result<Vec<AccountMeta>> {
        let accs = match self {
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
                (match self {
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

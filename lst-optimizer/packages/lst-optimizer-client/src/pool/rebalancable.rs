use anyhow::Result;
use controller_lib::{
    calculator::typedefs::CalculatorType, rebalance::RebalancingInstructions, Pubkey,
};
use jupiter_lib::quoter::JupiterInstructionBuilder;
use log::{info, warn};
use lst_optimizer_std::{
    pool::PoolRebalancable,
    types::{
        amount_change::AmountChange, context::Context, pool_allocation_changes::PoolAssetChange,
    },
};
use solana_sdk::instruction::Instruction;
use spl_helper::token_account::TokenAccountQuery;
use spl_token::native_mint;

use crate::typedefs::pool_to_calculator_type;

use super::pool::MaxPool;

#[async_trait::async_trait]
impl PoolRebalancable for MaxPool {
    async fn rebalance_asset(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        let asset = context.get_asset_from_mint(&pool_asset_change.mint)?;
        let (src_mint, dst_mint, src_cal, dst_cal, amount) = match pool_asset_change.amount {
            AmountChange::Increase(amt) => (
                native_mint::ID,
                pool_asset_change.mint.parse()?,
                CalculatorType::Wsol,
                pool_to_calculator_type(&asset)?,
                amt,
            ),
            AmountChange::Decrease(amt) => (
                pool_asset_change.mint.parse()?,
                native_mint::ID,
                pool_to_calculator_type(&asset)?,
                CalculatorType::Wsol,
                amt,
            ),
        };

        if src_mint.eq(&dst_mint) {
            warn!("The source and destination mints are the same, no rebalance needed");
            return Ok(());
        }

        let payer: Pubkey = context.payer.parse()?;
        let controller = self.controller_client();
        let rpc = controller.rpc_client();

        let src_ata = src_mint
            .resolve_associated_token_account(&payer, rpc)
            .await?;

        let program_id = self.program_id();

        let mut instructions: Vec<Instruction> = vec![];
        let start_ix = controller
            .create_start_rebalance_instruction(
                &program_id,
                &src_ata,
                &src_mint,
                &dst_mint,
                src_cal.clone(),
                dst_cal.clone(),
                amount,
            )
            .await?;

        let jup_client = JupiterInstructionBuilder::new();
        let swap_ixs = jup_client
            .create_jupiter_swap_instruction(&payer, &src_mint, &dst_mint, amount, None)
            .await?;

        let end_ix = controller
            .create_end_rebalance_instruction_from_start(&start_ix)
            .await?;

        instructions.extend(swap_ixs.compute_budget_instructions);
        instructions.push(start_ix);
        instructions.extend(swap_ixs.setup_instructions);
        instructions.push(swap_ixs.swap_instruction);
        instructions.push(end_ix);

        let address_lookup_table_accs = jup_client
            .get_address_lookup_table_accounts(rpc, swap_ixs.address_lookup_table_addresses)
            .await?;

        let ret = controller
            .simulate_instructions(&payer, &instructions, &address_lookup_table_accs)
            .await?;

        match ret.err {
            Some(err) => {
                warn!("Error: {}", err);
            }
            None => {
                info!("Rebalance successful");
            }
        };

        Ok(())
    }
}

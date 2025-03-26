use anyhow::Result;
use controller_lib::{rebalance::RebalancingInstructions, state::PoolQuery, Pubkey};
use log::{error, info, warn};
use lst_optimizer_std::{
    pool::PoolRebalancable,
    types::{context::Context, pool_allocation_changes::PoolAssetChange},
};
use solana_sdk::instruction::Instruction;
use spl_helper::token_account::TokenAccountQuery;

use crate::pool::helper::pool_asset_change_route::{PoolAssetChangeRoute, PoolAssetChangeRouter};

use super::pool::MaxPool;

#[async_trait::async_trait]
impl PoolRebalancable for MaxPool {
    async fn rebalance_asset(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        let asset = context.get_known_asset_from_mint(&pool_asset_change.mint)?;
        let PoolAssetChangeRoute {
            src_mint,
            dst_mint,
            src_cal,
            dst_cal,
            amount,
        } = pool_asset_change.get_route(&asset)?;

        if src_mint.eq(&dst_mint) {
            warn!("The source and destination mints are the same, no rebalance needed");
            return Ok(());
        }

        let payer: Pubkey = context.get_payer_pubkey();
        let controller = self.controller_client();
        let rpc = controller.rpc_client();
        let quoter_client = self.quoter_client();
        let program_id = self.program_id();

        let reserves_ata = controller
            .get_pool_reserves_address_by_mint(&program_id, &dst_mint)
            .await?;
        let swap_ixs = quoter_client
            .create_swap_instructions(&payer, &reserves_ata, &src_mint, &dst_mint, amount, 0, None)
            .await?;
        let address_lookup_table_accs = quoter_client
            .resolve_address_lookup_table_accounts(swap_ixs.address_lookup_tables)
            .await?;

        // prepare the accounts if needed
        // returned if setup instructions failed
        // if swap_ixs.setup_instructions.len() > 0 {
        //     info!("Invoking setup instructions");
        //     let signature = controller
        //         .invoke_instructions(
        //             context.get_payer(),
        //             &swap_ixs.setup_instructions,
        //             &address_lookup_table_accs,
        //         )
        //         .await?;
        //     info!("Setup invoked with signature: {}", signature)
        // }

        // rebalance instructions
        let mut instructions: Vec<Instruction> = vec![];
        let src_ata = src_mint
            .resolve_associated_token_account(&payer, rpc)
            .await?;

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
        let end_ix = controller
            .create_end_rebalance_instruction_from_start(&start_ix)
            .await?;

        // instructions.extend(swap_ixs.compute_budget_instructions);
        instructions.push(start_ix);
        instructions.extend(swap_ixs.setup_instructions);
        instructions.extend(swap_ixs.swap_instructions);
        instructions.push(end_ix);

        info!("Invoking rebalance instructions");
        let ret = controller
            .invoke_instructions(
                context.get_payer(),
                &instructions,
                &address_lookup_table_accs,
            )
            .await;
        match ret {
            Ok(signature) => {
                info!("Rebalance invoked with signature: {}", signature);

                if swap_ixs.cleanup_instructions.len() > 0 {
                    info!("Invoking cleanup instructions");
                    let ret = controller
                        .invoke_instructions(
                            context.get_payer(),
                            &swap_ixs.cleanup_instructions,
                            &address_lookup_table_accs,
                        )
                        .await;
                    match ret {
                        Ok(signature) => info!("Cleanup invoked with signature: {}", signature),
                        Err(e) => error!("Cleanup failed with error: {}", e),
                    }
                }
            }
            Err(e) => error!("Rebalance failed with error: {}", e),
        }

        Ok(())
    }
}

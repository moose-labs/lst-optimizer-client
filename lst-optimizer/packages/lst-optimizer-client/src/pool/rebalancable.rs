use anyhow::Result;
use controller_lib::{rebalance::RebalancingInstructions, state::PoolQuery, Pubkey};
use log::{info, warn};
use lst_optimizer_std::{
    pool::PoolRebalancable,
    types::{context::Context, pool_allocation_changes::PoolAssetChange},
};
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_helper::{mint::MintAccountQuery, token_account::TokenAccountQuery};

use crate::pool::helper::{
    pool_asset_change_route::{PoolAssetChangeRoute, PoolAssetChangeRouter},
    transaction_err::handle_error,
};

use super::pool::MaxPool;

#[async_trait::async_trait]
impl PoolRebalancable for MaxPool {
    async fn rebalance_asset(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        let asset = context.get_known_asset_from_mint(&pool_asset_change.mint)?;
        info!("Rebalancing asset: {}", asset.symbol);

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
        let pool_program_id = self.program_id();

        let reserves_ata = controller
            .get_pool_reserves_address_by_mint(&pool_program_id, &dst_mint)
            .await?;
        let swap_ixs = quoter_client
            .create_swap_instructions(&payer, &reserves_ata, &src_mint, &dst_mint, amount, 0, None)
            .await?;
        let address_lookup_table_accs = quoter_client
            .resolve_address_lookup_table_accounts(swap_ixs.address_lookup_tables)
            .await?;

        // rebalance instructions
        let mut instructions: Vec<Instruction> = vec![];

        let mint_program_id = src_mint.get_mint_owner(rpc).await?;
        let src_ata = src_mint
            .get_associated_token_account_with_program_id(&payer, &mint_program_id)
            .await?;
        let src_ata_acc = rpc.get_account(&src_ata).await;
        if src_ata_acc.is_err() {
            instructions.push(create_associated_token_account_idempotent(
                &payer,
                &payer,
                &src_mint,
                &mint_program_id,
            ));
        }

        let start_ix = controller
            .create_start_rebalance_instruction(
                &pool_program_id,
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

        instructions.push(start_ix);
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
                        Err(e) => handle_error(e),
                    }
                }
            }
            Err(e) => handle_error(e),
        }

        Ok(())
    }
}

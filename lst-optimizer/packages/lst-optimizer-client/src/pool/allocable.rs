use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use controller_lib::{
    calculator::{query::CalculatorQuery, typedefs::CalculatorType},
    rebalance::RebalancingInstructions,
    state::PoolQuery,
    Pubkey,
};
use log::{info, warn};
use lst_optimizer_std::{
    allocator::AllocationRatios,
    pool::Pool,
    types::{
        amount_change::AmountChange,
        context::Context,
        pool_allocation::PoolAllocations,
        pool_allocation_changes::{
            PoolAllocationChanges, PoolAllocationLamportsChanges, PoolAssetChange,
            PoolAssetLamportsChange,
        },
        pool_asset::PoolAsset,
    },
};
use spl_helper::{mint::MintAccountQuery, token_account::TokenAccountQuery};

use crate::typedefs::pool_to_calculator_type;

use super::pool::MaxPool;
use spl_token::native_mint;

#[async_trait::async_trait]
impl Pool for MaxPool {
    async fn get_allocation(&self, context: &Context) -> Result<PoolAllocations> {
        let controller: Pubkey = self.program_id();
        let controller_client = self.controller_client();
        let pool_state_addr = controller_client.get_pool_state_address(&controller).await;
        let lst_state_list = controller_client
            .get_lst_state_list_from_program_id(&controller)
            .await?;

        let mut assets: Vec<PoolAsset> = vec![];
        for lst_state in lst_state_list {
            let known_asset = context.get_asset_from_mint(&lst_state.mint.to_string())?;
            let token_program: Pubkey = known_asset.token_program.parse()?;

            let pool_reserves_address = controller_client
                .get_pool_reserves_address(&lst_state, &pool_state_addr, &token_program)
                .await?;
            let reserves = controller_client
                .get_pool_reserves_account(&pool_reserves_address)
                .await?;

            assets.push(PoolAsset::new(
                &lst_state.mint.to_string(),
                lst_state.sol_value,
                reserves.amount,
            ));
        }
        Ok(PoolAllocations { assets: assets })
    }

    async fn get_allocation_lamports_changes(
        &self,
        _: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios,
    ) -> Result<PoolAllocationLamportsChanges> {
        let total_lamports = pool_allocations.get_total_lamports();

        let mut target_lamports_per_symbol: HashMap<String, u64> = HashMap::new();
        for symbol_ratio in new_allocation_ratios.asset_alloc_ratios.iter() {
            target_lamports_per_symbol.insert(
                symbol_ratio.mint.to_owned(),
                self.calculate_lamports_from_bps(total_lamports, symbol_ratio.bps)?,
            );
        }

        // Calculate allocation changes
        let mut changes: HashMap<String, AmountChange> = HashMap::new();
        for (mint, target_lamports) in target_lamports_per_symbol.iter() {
            let current_allocation = pool_allocations.get_pool_asset(mint);
            let target_lamports = *target_lamports;
            let current_lamports = match current_allocation {
                Some(allocation) => allocation.lamports,
                None => 0,
            };

            // target_lamports > current_lamports (increase)
            if target_lamports > current_lamports {
                let lamports_change = target_lamports - current_lamports;
                changes.insert(mint.clone(), AmountChange::Increase(lamports_change));
            } else if target_lamports < current_lamports {
                let lamports_change = current_lamports - target_lamports;
                changes.insert(mint.clone(), AmountChange::Decrease(lamports_change));
            }
        }

        // Add current allocations that are not in the new allocation ratios
        let current_assets = &pool_allocations.assets;
        for asset in current_assets.iter() {
            let mint = &asset.mint;
            if !changes.contains_key(mint) {
                changes.insert(mint.clone(), AmountChange::Decrease(asset.lamports));
            }
        }

        Ok(PoolAllocationLamportsChanges {
            assets: changes
                .iter()
                .map(|(mint, lamports_change)| {
                    PoolAssetLamportsChange::new(mint, lamports_change.clone())
                })
                .collect(),
        })
    }

    async fn get_allocation_changes(
        &self,
        context: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios,
    ) -> Result<PoolAllocationChanges> {
        let payer: Pubkey = context.payer.parse()?;
        let controller = self.controller_client();
        let pool_options = self.pool_options();

        let changes = self
            .get_allocation_lamports_changes(context, pool_allocations, new_allocation_ratios)
            .await?;

        let mut asset_changes: Vec<PoolAssetChange> = vec![];
        for asset_lamports_change in changes.assets.iter() {
            let mint = &asset_lamports_change.mint;
            let lamports_change = match asset_lamports_change.lamports {
                AmountChange::Increase(amount) => amount,
                AmountChange::Decrease(amount) => amount,
            };

            let known_asset = context.get_asset_from_mint(mint)?;
            let calculator_type = pool_to_calculator_type(&known_asset)?;

            let mut reserves_change = 0 as u64;
            if lamports_change > pool_options.minimum_rebalance_lamports {
                let reserves_change_range = controller
                    .convert_sol_to_lst(&payer, calculator_type, lamports_change)
                    .await?;
                reserves_change = reserves_change_range.get_min();
            } else {
                warn!(
                    "The amount of lamports ({}) to rebalance is less than the minimum rebalance lamports ({})",
                    lamports_change,
                    pool_options.minimum_rebalance_lamports
                );
            }

            let asset_change = match asset_lamports_change.lamports {
                AmountChange::Increase(_) => {
                    PoolAssetChange::new(mint, AmountChange::Increase(reserves_change))
                }
                AmountChange::Decrease(_) => {
                    PoolAssetChange::new(mint, AmountChange::Decrease(reserves_change))
                }
            };

            asset_changes.push(asset_change);
        }

        Ok(PoolAllocationChanges {
            assets: asset_changes,
        })
    }

    async fn rebalance_asset(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        let asset = context.get_asset_from_mint(&pool_asset_change.mint)?;
        let (src_mint, dst_mint, src_cal, dst_cal, amount) = match pool_asset_change.amount {
            AmountChange::Increase(amt) => (
                native_mint::ID,
                Pubkey::from_str(pool_asset_change.mint.as_str())?,
                CalculatorType::Wsol,
                pool_to_calculator_type(&asset)?,
                amt,
            ),
            AmountChange::Decrease(amt) => (
                Pubkey::from_str(pool_asset_change.mint.as_str())?,
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

        let subsidized_ata = dst_mint
            .resolve_associated_token_account(&payer, rpc)
            .await?;

        // get balance
        let balance = subsidized_ata.get_token_account_balance(rpc).await?;
        println!("token account: {}", subsidized_ata);
        println!("balance: {}", balance);
        println!("subsidized amount: {}", amount);

        let pool_reserve_addr = controller
            .find_pool_reserves_address(
                &controller.get_pool_state_address(&program_id).await,
                &dst_mint,
                &dst_mint.get_mint_owner(rpc).await?,
            )
            .await?;
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::ID,
            &subsidized_ata,
            &pool_reserve_addr,
            &payer,
            &[&payer],
            amount,
        )?;

        let end_ix = controller
            .create_end_rebalance_instruction_from_start(&start_ix)
            .await?;

        // let swap_ix = controller
        //     .create_sanctum_swap_exact_in_instruction(
        //         &program_id,
        //         &payer,
        //         &src_mint
        //             .resolve_associated_token_account(&payer, rpc)
        //             .await?,
        //         &dst_mint
        //             .resolve_associated_token_account(&payer, rpc)
        //             .await?,
        //         &src_mint,
        //         &dst_mint,
        //         src_cal,
        //         dst_cal,
        //         amount,
        //         0,
        //     )
        //     .await?;

        let ret = controller
            .simulate_instructions(&payer, &[start_ix, transfer_ix, end_ix], &[])
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

#[cfg(test)]
mod tests {
    use lst_optimizer_std::allocator::AllocationRatio;

    use crate::pool::{pool::MaxPool, typedefs::MaxPoolOptions};

    use super::*;

    #[tokio::test]
    async fn test_get_allocation_changes() {
        let pool = MaxPool::new("", MaxPoolOptions::default());
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("hsol", 400, 0),
                PoolAsset::new("jitosol", 100, 0),
                PoolAsset::new("jupsol", 0, 0),
                PoolAsset::new("inf", 0, 0),
            ],
        };
        let new_allocation_ratios = AllocationRatios::new(vec![
            AllocationRatio::new("jupsol", 5000),
            AllocationRatio::new("inf", 5000),
        ]);
        let changes = pool
            .get_allocation_lamports_changes(
                &Context::default(),
                &pool_allocations,
                &new_allocation_ratios,
            )
            .await
            .unwrap();
        assert_eq!(changes.assets.len(), 4);
        assert_eq!(
            changes.get_asset_lamports_changes("hsol").unwrap().lamports,
            AmountChange::Decrease(400)
        );
        assert_eq!(
            changes
                .get_asset_lamports_changes("jitosol")
                .unwrap()
                .lamports,
            AmountChange::Decrease(100)
        );
        assert_eq!(
            changes
                .get_asset_lamports_changes("jupsol")
                .unwrap()
                .lamports,
            AmountChange::Increase(250)
        );
        assert_eq!(
            changes.get_asset_lamports_changes("inf").unwrap().lamports,
            AmountChange::Increase(250)
        );
    }
}

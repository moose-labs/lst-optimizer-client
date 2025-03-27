use std::collections::HashMap;

use anyhow::Result;
use controller_lib::{calculator::query::CalculatorQuery, state::PoolQuery, Pubkey};
use log::warn;
use lst_optimizer_std::{
    allocator::AllocationRatios,
    pool::PoolAllocable,
    types::{
        amount_change::AmountChange,
        context::Context,
        lamports_change::LamportsChange,
        pool_allocation::PoolAllocations,
        pool_allocation_changes::{
            PoolAllocationChanges, PoolAllocationLamportsChanges, PoolAssetChange,
            PoolAssetLamportsChange,
        },
        pool_asset::PoolAsset,
    },
};

use crate::typedefs::pool_to_calculator_type;

use super::pool::MaxPool;

#[async_trait::async_trait]
impl PoolAllocable for MaxPool {
    async fn get_allocation(&self, context: &Context) -> Result<PoolAllocations> {
        let controller: Pubkey = self.program_id();
        let controller_client = self.controller_client();
        let pool_state_addr = controller_client.get_pool_state_address(&controller).await;
        let lst_state_list = controller_client
            .get_lst_state_list_from_program_id(&controller)
            .await?;

        let mut assets: Vec<PoolAsset> = vec![];
        for lst_state in lst_state_list {
            let known_asset = context.get_known_asset_from_mint(&lst_state.mint.to_string())?;
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
        let pool_options = self.pool_options();
        let total_lamports = pool_allocations.get_total_lamports();

        let mut target_lamports_per_symbol: HashMap<String, u64> = HashMap::new();
        for symbol_ratio in new_allocation_ratios.asset_alloc_ratios.iter() {
            target_lamports_per_symbol.insert(
                symbol_ratio.mint.to_owned(),
                self.calculate_lamports_from_bps(total_lamports, symbol_ratio.bps)?,
            );
        }

        // Calculate allocation changes
        let mut changes: HashMap<String, LamportsChange> = HashMap::new();
        for (mint, target_lamports) in target_lamports_per_symbol.iter() {
            let current_allocation = pool_allocations.get_pool_asset(mint);
            let target_lamports = *target_lamports;
            let current_lamports = match current_allocation {
                Some(allocation) => allocation.lamports,
                None => 0,
            };

            // target_lamports > current_lamports (increase)
            let maximum_rebalance_lamports = pool_options.maximum_rebalance_lamports;
            let mut lamports_change = current_lamports.abs_diff(target_lamports);
            if lamports_change > maximum_rebalance_lamports {
                lamports_change = maximum_rebalance_lamports;
            }
            if target_lamports > current_lamports {
                changes.insert(mint.clone(), LamportsChange::Increase(lamports_change));
            } else if target_lamports < current_lamports {
                changes.insert(mint.clone(), LamportsChange::Decrease(lamports_change));
            }
        }

        // Add current allocations that are not in the new allocation ratios
        let current_assets = &pool_allocations.assets;
        for asset in current_assets.iter() {
            let mint = &asset.mint;
            if !changes.contains_key(mint) {
                changes.insert(mint.clone(), LamportsChange::Decrease(asset.lamports));
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
        let controller = self.controller_client();
        let pool_options = self.pool_options();

        let changes = self
            .get_allocation_lamports_changes(context, pool_allocations, new_allocation_ratios)
            .await?;

        let mut asset_changes: Vec<PoolAssetChange> = vec![];
        for asset_lamports_change in changes.assets.iter() {
            let mint = &asset_lamports_change.mint;
            let lamports_change = asset_lamports_change.lamports.get_lamports();

            let known_asset = context.get_known_asset_from_mint(mint)?;
            let calculator_type = pool_to_calculator_type(&known_asset)?;

            if lamports_change > pool_options.minimum_rebalance_lamports {
                let lst_change_range = controller
                    .convert_sol_to_lst(context.get_payer(), calculator_type, lamports_change)
                    .await?;
                let lst_change = lst_change_range.get_min();
                let asset_change = match asset_lamports_change.lamports {
                    LamportsChange::Increase(_) => PoolAssetChange::new(
                        mint,
                        AmountChange::Increase {
                            lamports: lamports_change,
                            lst_amount: lst_change,
                        },
                    ),
                    LamportsChange::Decrease(_) => PoolAssetChange::new(
                        mint,
                        AmountChange::Decrease {
                            lamports: lamports_change,
                            lst_amount: lst_change,
                        },
                    ),
                };
                asset_changes.push(asset_change);
            } else {
                warn!(
                    "The amount of lamports ({}) to rebalance is less than the minimum rebalance lamports ({})",
                    lamports_change,
                    pool_options.minimum_rebalance_lamports
                );
            }
        }

        Ok(PoolAllocationChanges {
            assets: asset_changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use lst_optimizer_std::allocator::AllocationRatio;
    use quoter_lib::mock_quoter::MockQuoterClient;

    use crate::pool::{pool::MaxPool, typedefs::MaxPoolOptions};

    use super::*;

    #[tokio::test]
    async fn test_get_allocation_lamports_changes() {
        let pool = MaxPool::new(
            Pubkey::new_unique(),
            Box::new(MockQuoterClient::new()),
            MaxPoolOptions::default(),
        );
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
            LamportsChange::Decrease(400)
        );
        assert_eq!(
            changes
                .get_asset_lamports_changes("jitosol")
                .unwrap()
                .lamports,
            LamportsChange::Decrease(100)
        );
        assert_eq!(
            changes
                .get_asset_lamports_changes("jupsol")
                .unwrap()
                .lamports,
            LamportsChange::Increase(250)
        );
        assert_eq!(
            changes.get_asset_lamports_changes("inf").unwrap().lamports,
            LamportsChange::Increase(250)
        );
    }

    #[tokio::test]
    async fn test_maximum_rebalancing_lamports() {
        let pool = MaxPool::new(
            Pubkey::new_unique(),
            Box::new(MockQuoterClient::new()),
            MaxPoolOptions {
                maximum_rebalance_lamports: 200,
                ..Default::default()
            },
        );
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("hsol", 700, 0),
                PoolAsset::new("jitosol", 100, 0),
            ],
        };
        let new_allocation_ratios = AllocationRatios::new(vec![
            AllocationRatio::new("hsol", 5000),
            AllocationRatio::new("jitosol", 5000),
        ]);
        let changes = pool
            .get_allocation_lamports_changes(
                &Context::default(),
                &pool_allocations,
                &new_allocation_ratios,
            )
            .await
            .unwrap();

        // should rebalance to 400 each
        assert_eq!(
            changes.get_asset_lamports_changes("hsol").unwrap().lamports,
            LamportsChange::Decrease(200) // -300
        );
        assert_eq!(
            changes
                .get_asset_lamports_changes("jitosol")
                .unwrap()
                .lamports,
            LamportsChange::Increase(200) // +300
        );
    }
}

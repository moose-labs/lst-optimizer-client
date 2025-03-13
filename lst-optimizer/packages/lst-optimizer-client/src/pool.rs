use std::collections::HashMap;

use controller_lib::{
    calculator::convert_sol_to_lst,
    state::{
        find_and_get_lst_state_list,
        find_pool_reserves_address,
        find_pool_state_address,
        get_reserves_account,
        LstState,
    },
    Pubkey,
};
use lst_optimizer_std::{
    allocator::AllocationRatios,
    pool::{ Pool, PoolError },
    types::{
        amount_change::AmountChange,
        asset::Asset,
        context::Context,
        pool_allocation::{ PoolAllocations, MAX_ALLOCATION_BPS },
        pool_allocation_changes::{
            PoolAllocationChanges,
            PoolAllocationLamportsChanges,
            PoolAssetChange,
            PoolAssetLamportsChange,
        },
        pool_asset::PoolAsset,
    },
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use anyhow::{ Context as _AnyhowContext, Ok, Result };
use solana_client::rpc_client::RpcClient;

use crate::typedefs::pool_to_calculator_type;

#[derive(Debug, Clone)]
pub struct MaxPoolOptions {
    pub rpc_url: String,
}

impl Default for MaxPoolOptions {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaxPool {
    // A controller program id
    program_id: String,
    options: MaxPoolOptions,
}

impl MaxPool {
    pub fn new(program_id: &str, options: Option<MaxPoolOptions>) -> Self {
        Self {
            program_id: program_id.to_string(),
            options: options.unwrap_or(MaxPoolOptions::default()),
        }
    }

    fn get_rpc(&self) -> RpcClient {
        RpcClient::new(self.options.rpc_url.clone())
    }

    fn calculate_lamports_per_symbol(
        &self,
        total_lamports: u64,
        symbol_bps: Decimal
    ) -> Result<u64> {
        let total_lamports_dec = Decimal::from(total_lamports);
        let max_bps = Decimal::from(MAX_ALLOCATION_BPS);
        let ratio = symbol_bps
            .checked_div(max_bps)
            .context(PoolError::FailedToCalculateLamportsPerSymbol(total_lamports, symbol_bps))?;

        let target_lamports = ratio
            .checked_mul(total_lamports_dec)
            .context(PoolError::FailedToCalculateAllocationChanges)?;

        Ok(
            target_lamports
                .ceil()
                .to_u64()
                .ok_or(PoolError::FailedToCalculateLamportsPerSymbol(total_lamports, symbol_bps))?
        )
    }

    fn get_lst_state(&self) -> Result<Vec<LstState>> {
        let rpc = self.get_rpc();
        let program_id = self.program_id.parse()?;
        let lst_state_list = find_and_get_lst_state_list(&rpc, &program_id)?;
        Ok(lst_state_list)
    }

    fn get_lst_reserves_balance(
        &self,
        lst_state: &LstState,
        controller: &Pubkey,
        token_program: &Pubkey
    ) -> Result<u64> {
        let rpc = self.get_rpc();
        let pool_state_address = find_pool_state_address(controller);
        let reserves_addr = find_pool_reserves_address(
            lst_state,
            &pool_state_address,
            token_program
        )?;
        let reserves_account = get_reserves_account(&rpc, &reserves_addr)?;
        Ok(reserves_account.amount)
    }
}

impl Pool for MaxPool {
    fn get_allocation(&self, context: &Context) -> Result<PoolAllocations> {
        let controller: Pubkey = self.program_id.parse()?;
        let lst_state_list = self.get_lst_state()?;
        let mut assets: Vec<PoolAsset> = vec![];
        for lst_state in lst_state_list {
            let mint = lst_state.mint;
            let lamports = lst_state.sol_value;
            let known_asset = context.get_asset(&mint.to_string());
            // TODO: remove this assertion
            if known_asset.is_err() {
                continue;
            }
            let asset: Asset = known_asset.unwrap();
            let token_program: Pubkey = asset.token_program.parse()?;
            let reserves = self.get_lst_reserves_balance(&lst_state, &controller, &token_program)?;
            assets.push(PoolAsset::new(&mint.to_string(), lamports, reserves));
        }
        Ok(PoolAllocations {
            assets: assets,
        })
    }

    fn get_allocation_lamports_changes(
        &self,
        _context: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios
    ) -> Result<PoolAllocationLamportsChanges> {
        let total_lamports = pool_allocations.get_total_lamports();

        // Calculate target lamports per symbol
        let mut target_lamports_per_symbol: HashMap<String, u64> = HashMap::new();
        for symbol_ratio in new_allocation_ratios.asset_alloc_ratios.iter() {
            let symbol_bps = symbol_ratio.bps;
            let target_lamports = self.calculate_lamports_per_symbol(total_lamports, symbol_bps)?;
            target_lamports_per_symbol.insert(symbol_ratio.mint.clone(), target_lamports);
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
                .map(|(mint, lamports_change)|
                    PoolAssetLamportsChange::new(mint, lamports_change.clone())
                )
                .collect(),
        })
    }

    fn get_allocation_changes(
        &self,
        context: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios
    ) -> Result<PoolAllocationChanges> {
        let changes = self.get_allocation_lamports_changes(
            context,
            pool_allocations,
            new_allocation_ratios
        )?;

        let payer: Pubkey = context.payer.parse()?;
        let rpc = self.get_rpc();
        let mut asset_changes: Vec<PoolAssetChange> = vec![];
        for asset_lamports_change in changes.assets.iter() {
            let mint = &asset_lamports_change.mint;
            let lamports_change = match asset_lamports_change.lamports {
                AmountChange::Increase(amount) => amount,
                AmountChange::Decrease(amount) => amount,
            };

            let known_asset = context.get_asset(mint)?;
            let calculator_type = pool_to_calculator_type(&known_asset)?;
            let reserves_change = convert_sol_to_lst(
                &rpc,
                &payer,
                calculator_type,
                lamports_change
            )?;

            let reserves_change = reserves_change.get_min();
            let asset_change = match asset_lamports_change.lamports {
                AmountChange::Increase(_) =>
                    PoolAssetChange::new(mint, AmountChange::Increase(reserves_change)),
                AmountChange::Decrease(_) =>
                    PoolAssetChange::new(mint, AmountChange::Decrease(reserves_change)),
            };

            asset_changes.push(asset_change);
        }

        Ok(PoolAllocationChanges {
            assets: asset_changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use lst_optimizer_std::allocator::AllocationRatio;

    use super::*;

    #[test]
    fn test_calculate_lamports_per_symbol_success() {
        let pool = MaxPool::new("", None);
        let total_lamports = 1_000_000;
        let symbol_bps = Decimal::from(5000);
        let target_lamports = pool
            .calculate_lamports_per_symbol(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 500_000);
    }

    #[test]
    fn test_calculate_lamports_per_symbol_success_on_division_by_zeros() {
        let pool = MaxPool::new("", None);

        // symbol_bps = 0
        let total_lamports = 1_000_000;
        let symbol_bps = Decimal::from(0);
        let target_lamports = pool
            .calculate_lamports_per_symbol(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 0);

        // total_lamports = 0
        let total_lamports = 0;
        let symbol_bps = Decimal::from(1000);
        let target_lamports = pool
            .calculate_lamports_per_symbol(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 0);
    }

    #[test]
    fn test_get_allocation_changes() {
        let pool = MaxPool::new("", None);
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("hsol", 400, 0),
                PoolAsset::new("jitosol", 100, 0),
                PoolAsset::new("jupsol", 0, 0),
                PoolAsset::new("inf", 0, 0)
            ],
        };
        let new_allocation_ratios = AllocationRatios::new(
            vec![AllocationRatio::new("jupsol", 5000), AllocationRatio::new("inf", 5000)]
        );
        let changes = pool
            .get_allocation_lamports_changes(
                &Context::default(),
                &pool_allocations,
                &new_allocation_ratios
            )
            .unwrap();
        assert_eq!(changes.assets.len(), 4);
        assert_eq!(
            changes.get_asset_lamports_changes("hsol").unwrap().lamports,
            AmountChange::Decrease(400)
        );
        assert_eq!(
            changes.get_asset_lamports_changes("jitosol").unwrap().lamports,
            AmountChange::Decrease(100)
        );
        assert_eq!(
            changes.get_asset_lamports_changes("jupsol").unwrap().lamports,
            AmountChange::Increase(250)
        );
        assert_eq!(
            changes.get_asset_lamports_changes("inf").unwrap().lamports,
            AmountChange::Increase(250)
        );
    }
}

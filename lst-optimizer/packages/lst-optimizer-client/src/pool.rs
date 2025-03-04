use std::collections::HashMap;

use log::info;
use lst_optimizer_std::{
    allocator::AllocationRatios,
    pool::{ Pool, PoolError },
    types::{
        pool_allocation::{ PoolAllocations, MAX_ALLOCATION_BPS },
        pool_allocation_changes::PoolAllocationChanges,
        pool_asset::PoolAsset,
    },
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use anyhow::{ Context, Ok, Result };

#[derive(Debug, Clone)]
pub struct MaxPool {
    _address: String,
}

impl MaxPool {
    pub fn new(address: &str) -> Self {
        Self {
            _address: address.to_string(),
        }
    }

    fn calculate_lamports_per_symbol(
        &self,
        total_lamports: i128,
        symbol_bps: Decimal
    ) -> Result<i128> {
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
                .to_i128()
                .ok_or(PoolError::FailedToCalculateLamportsPerSymbol(total_lamports, symbol_bps))?
        )
    }
}

impl Pool for MaxPool {
    fn get_allocation(&self) -> Result<PoolAllocations> {
        // TODO: fetch on-chain allocations
        let lamports = 1_000_000_000_000 as i128;
        Ok(PoolAllocations {
            assets: vec![PoolAsset::new("hsol", lamports), PoolAsset::new("jitosol", lamports)],
        })
    }

    fn get_allocation_changes(
        &self,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios
    ) -> Result<PoolAllocationChanges> {
        let total_lamports = pool_allocations.get_total_lamports();

        // Calculate target lamports per symbol
        let mut target_lamports_per_symbol: HashMap<String, i128> = HashMap::new();
        for symbol_ratio in new_allocation_ratios.asset_alloc_ratios.iter() {
            let symbol_bps = symbol_ratio.bps;
            let target_lamports = self.calculate_lamports_per_symbol(total_lamports, symbol_bps)?;
            target_lamports_per_symbol.insert(symbol_ratio.symbol.clone(), target_lamports);
        }
        info!("calculate target lamports per symbol: {:?}", target_lamports_per_symbol);

        // TODO: calculate exchange rates

        // Calculate allocation changes
        let mut changes: HashMap<String, i128> = HashMap::new();
        for (symbol, target_lamports) in target_lamports_per_symbol.iter() {
            let current_allocation = pool_allocations.get_pool_asset(symbol);
            let current_lamports = match current_allocation {
                Some(allocation) => allocation.lamports,
                None => 0,
            };
            let lamports_change = *target_lamports - current_lamports;
            changes.insert(symbol.clone(), lamports_change);
        }

        // Add current allocations that are not in the new allocation ratios
        let current_assets = &pool_allocations.assets;
        for asset in current_assets.iter() {
            let symbol = &asset.symbol;
            if !changes.contains_key(symbol) {
                changes.insert(symbol.clone(), -asset.lamports);
            }
        }
        info!("calculate changes: {:?}", changes);

        Ok(PoolAllocationChanges {
            assets: changes
                .iter()
                .map(|(symbol, lamports_change)| PoolAsset::new(symbol, *lamports_change))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use lst_optimizer_std::allocator::AllocationRatio;

    use super::*;

    #[test]
    fn test_calculate_lamports_per_symbol_success() {
        let pool = MaxPool::new("");
        let total_lamports = 1_000_000;
        let symbol_bps = Decimal::from(5000);
        let target_lamports = pool
            .calculate_lamports_per_symbol(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 500_000);
    }

    #[test]
    fn test_calculate_lamports_per_symbol_success_on_division_by_zeros() {
        let pool = MaxPool::new("");

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
        let pool = MaxPool::new("");
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("hsol", 400),
                PoolAsset::new("jitosol", 100),
                PoolAsset::new("jupsol", 0),
                PoolAsset::new("inf", 0)
            ],
        };
        let new_allocation_ratios = AllocationRatios::new(
            vec![AllocationRatio::new("jupsol", 5000), AllocationRatio::new("inf", 5000)]
        );
        let changes = pool
            .get_allocation_changes(&pool_allocations, &new_allocation_ratios)
            .unwrap();
        assert_eq!(changes.assets.len(), 4);
        assert_eq!(changes.get_asset_changes("hsol").unwrap().lamports, -400);
        assert_eq!(changes.get_asset_changes("jitosol").unwrap().lamports, -100);
        assert_eq!(changes.get_asset_changes("jupsol").unwrap().lamports, 250);
        assert_eq!(changes.get_asset_changes("inf").unwrap().lamports, 250);
    }
}

use core::time;

use anyhow::Result;
use log::debug;
use lst_optimizer_std::{
    allocator::Allocator,
    fetcher::fetcher::Fetcher,
    pool::Pool,
    types::{ context::Context, datapoint::SymbolData },
};

use crate::{
    allocator::ema::EmaAllocator,
    fetcher::apy::SanctumHistoricalApyFetcher,
    pool::MaxPool,
};

#[derive(Debug, Clone)]
pub struct OptimizerApp {
    program_id: String,
}

impl OptimizerApp {
    pub fn new(program_id: String) -> Self {
        Self {
            program_id,
        }
    }

    pub async fn keep_rebalance(&self, context: Context, interval: time::Duration) -> Result<()> {
        loop {
            self.rebalance(&context).await?;
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn rebalance(&self, context: &Context) -> Result<()> {
        let assets = context.asset_repository.get_assets();

        // Fetch historical APY data from the Sanctum API
        let fetcher = SanctumHistoricalApyFetcher::new();
        let mut symbol_datas = vec![];
        for asset in &assets {
            let datapoints = fetcher.fetch(&asset).await?;
            symbol_datas.push(SymbolData {
                mint: asset.mint.clone(),
                symbol: asset.symbol.clone(),
                datapoints,
            });
        }

        // Allocate assets and validate the allocations
        let allocator = EmaAllocator::new(Some(2), Some(5));
        let mut allocations = allocator.allocate(symbol_datas)?;
        allocations.validate()?;

        // Apply weights to the allocations and validate the allocations
        allocations.apply_weights(&assets);
        allocations.validate()?;

        // Get the current pool allocations and calculate the changes
        let pool = MaxPool::new(&self.program_id, None);
        let current_pool_allocations = pool.get_allocation(context)?;

        debug!("Pool allocation:");
        for pool_asset in current_pool_allocations.assets.iter() {
            debug!(" - pool asset: {:?}", pool_asset);
        }

        // Ensure that all pool assets are defined in the asset list
        // Otherwise, the optimizer should add the missing pool assets before rebalancing
        // TODO: enable assertion
        // current_pool_allocations.assert_pool_allocations_are_defined(&assets)?;

        // Get the allocation changes
        let pool_allocation_lamports_changes = pool.get_allocation_lamports_changes(
            context,
            &current_pool_allocations,
            &allocations
        )?;
        debug!("Pool allocation changes:");
        for pool_asset_change in pool_allocation_lamports_changes.assets.iter() {
            debug!(" - pool allocation changes (lamports): {:?}", pool_asset_change);
        }

        // Get the pool allocation changes
        let pool_allocation_changes = pool.get_allocation_changes(
            context,
            &current_pool_allocations,
            &allocations
        )?;
        debug!("Pool allocation changes:");
        for pool_asset_change in pool_allocation_changes.assets.iter() {
            debug!(" - pool allocation changes (lst): {:?}", pool_asset_change);
        }

        Ok(())
    }
}

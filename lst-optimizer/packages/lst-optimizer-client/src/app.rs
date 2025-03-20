use core::time;

use anyhow::Result;
use log::info;
use lst_optimizer_std::{
    allocator::Allocator,
    fetcher::fetcher::Fetcher,
    pool::{PoolAllocable, PoolRebalancable},
    types::{amount_change::AmountChange, context::Context, datapoint::SymbolData},
};

use crate::{
    allocator::ema::EmaAllocator, fetcher::apy::SanctumHistoricalApyFetcher, pool::pool::MaxPool,
};

pub struct OptimizerApp {
    pool: MaxPool,
}

impl OptimizerApp {
    pub fn new(pool: MaxPool) -> Self {
        Self { pool }
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

        let allocator = EmaAllocator::new(Some(2), Some(5));
        let mut allocations = allocator.allocate(symbol_datas)?;
        allocations.validate()?;
        allocations.apply_weights(&assets);
        allocations.validate()?;

        let pool = &self.pool;
        let current_pool_allocations = pool.get_allocation(context).await?;
        current_pool_allocations.assert_pool_allocations_are_defined(&assets)?;
        info!("{}", current_pool_allocations);

        let pool_allocation_lamports_changes = pool
            .get_allocation_lamports_changes(context, &current_pool_allocations, &allocations)
            .await?;
        info!("{}", pool_allocation_lamports_changes);

        let pool_allocation_changes = pool
            .get_allocation_changes(context, &current_pool_allocations, &allocations)
            .await?;
        info!("{}", pool_allocation_changes);

        // Reducing first
        for pool_asset_change in &pool_allocation_changes.assets {
            match pool_asset_change.amount {
                AmountChange::Increase(_) => {}
                AmountChange::Decrease(_) => {
                    let _ = pool.rebalance_asset(context, pool_asset_change).await?;
                }
            }
        }

        // Increasing
        for pool_asset_change in &pool_allocation_changes.assets {
            match pool_asset_change.amount {
                AmountChange::Increase(_) => {
                    let _ = pool.rebalance_asset(context, pool_asset_change).await?;
                }
                AmountChange::Decrease(_) => {}
            }
        }

        Ok(())
    }
}

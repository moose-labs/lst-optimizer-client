use core::time;

use anyhow::Result;

use log::{error, info};
use lst_optimizer_std::{
    allocator::{AllocationRatios, Allocator},
    fetcher::fetcher::Fetcher,
    pool::{PoolAllocable, PoolRebalancable},
    types::{
        amount_change::AmountChange,
        context::Context,
        datapoint::SymbolData,
        pool_allocation_changes::{PoolAllocationChanges, PoolAssetChange},
    },
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
            let res = self.rebalance(&context).await;
            if let Err(e) = res {
                info!("Failed to rebalance: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    }

    pub fn get_pool(&self) -> &MaxPool {
        &self.pool
    }

    /// Get the pool allocation changes based on the current pool allocations and the new allocation ratios
    ///
    pub async fn get_pool_allocation_changes(
        &self,
        context: &Context,
        allocations: AllocationRatios,
    ) -> Result<PoolAllocationChanges> {
        let assets = context.get_kwown_assets();
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

        Ok(pool_allocation_changes)
    }

    /// Retry rebalance pool asset change
    ///
    pub async fn try_rebalance_pool_asset_change(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        let ret = self.pool.rebalance_asset(context, pool_asset_change).await;
        if let Err(e) = ret {
            error!("Failed to rebalance pool asset change: {:?}", e);
        }

        tokio::time::sleep(time::Duration::from_secs(5)).await;

        Ok(())
    }

    pub async fn rebalance(&self, context: &Context) -> Result<()> {
        let assets = context.get_kwown_assets();

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

        let allocator = EmaAllocator::new(Some(10), Some(5));
        let mut allocations = allocator.allocate(symbol_datas)?;
        allocations.validate()?;
        allocations.apply_weights(&assets);
        allocations.validate()?;

        let pool_allocation_changes = self
            .get_pool_allocation_changes(context, allocations)
            .await?;

        // Reducing first
        for pool_asset_change in &pool_allocation_changes.assets {
            match pool_asset_change.amount {
                AmountChange::Increase { .. } => {}
                AmountChange::Decrease { .. } => {
                    self.try_rebalance_pool_asset_change(context, pool_asset_change)
                        .await?;
                }
            }
        }

        // Increasing
        for pool_asset_change in &pool_allocation_changes.assets {
            match pool_asset_change.amount {
                AmountChange::Increase { .. } => {
                    self.try_rebalance_pool_asset_change(context, pool_asset_change)
                        .await?;
                }
                AmountChange::Decrease { .. } => {}
            }
        }

        Ok(())
    }
}

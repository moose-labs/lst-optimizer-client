use core::time;

use anyhow::{Context as _AnyhowContext, Result};
use backoff::future::retry;
use log::info;
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
    allocator::ema::EmaAllocator, error::AppError, fetcher::apy::SanctumHistoricalApyFetcher,
    pool::pool::MaxPool, typedefs::default_backoff,
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
    pub async fn retry_rebalance_pool_asset_change(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()> {
        retry(default_backoff(), async || {
            let ret = self.pool.rebalance_asset(context, pool_asset_change).await;
            match ret {
                Ok(_) => Ok(()),
                Err(e) => Err(backoff::Error::transient(e)),
            }
        })
        .await
        .context(AppError::FailedToRetryRebalancePoolAssetChange)
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
                AmountChange::Increase(_) => {}
                AmountChange::Decrease(_) => {
                    let ret = self
                        .retry_rebalance_pool_asset_change(context, pool_asset_change)
                        .await;
                    if let Err(e) = ret {
                        info!("Failed to rebalance pool asset change: {:?}", e);
                    }
                }
            }
        }

        // Increasing
        for pool_asset_change in &pool_allocation_changes.assets {
            match pool_asset_change.amount {
                AmountChange::Increase(_) => {
                    let ret = self
                        .retry_rebalance_pool_asset_change(context, pool_asset_change)
                        .await;
                    if let Err(e) = ret {
                        info!("Failed to rebalance pool asset change: {:?}", e);
                    }
                }
                AmountChange::Decrease(_) => {}
            }
        }

        Ok(())
    }
}

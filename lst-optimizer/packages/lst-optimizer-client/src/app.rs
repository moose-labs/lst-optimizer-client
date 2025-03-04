use core::time;

use anyhow::Result;
use log::{ debug, info };
use lst_optimizer_std::{
    allocator::Allocator,
    fetcher::fetcher::Fetcher,
    pool::Pool,
    types::{ datapoint::SymbolData, asset::Asset },
};

use crate::{
    allocator::ema::EmaAllocator,
    fetcher::apy::SanctumHistoricalApyFetcher,
    pool::MaxPool,
};

#[derive(Debug, Clone)]
pub struct OptimizerApp {}

impl OptimizerApp {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn keep_rebalance(
        &self,
        assets: &Vec<Asset>,
        interval: time::Duration
    ) -> Result<()> {
        loop {
            self.rebalance(assets).await?;
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn rebalance(&self, assets: &Vec<Asset>) -> Result<()> {
        let fetcher = SanctumHistoricalApyFetcher::new();
        let mut symbol_datas = vec![];

        for asset in assets {
            let datapoints = fetcher.fetch(&asset.symbol).await?;
            symbol_datas.push(SymbolData {
                symbol: asset.symbol.clone(),
                datapoints,
            });
        }

        let allocator = EmaAllocator::new(Some(2), Some(5));
        let mut allocations = allocator.allocate(symbol_datas)?;
        allocations.validate()?;

        allocations.apply_weights(assets);
        allocations.validate()?;

        let pool = MaxPool::new("");
        let current_pool_allocations = pool.get_allocation()?;
        let pool_allocation_changes = pool.get_allocation_changes(
            &current_pool_allocations,
            &allocations
        )?;

        debug!("pool allocations: {:?}", current_pool_allocations);
        debug!("pool allocation changes: {:?}", pool_allocation_changes);

        Ok(())
    }
}

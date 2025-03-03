use core::time;

use anyhow::Result;
use log::info;
use lst_optimizer_std::{
    allocator::Allocator,
    fetcher::fetcher::Fetcher,
    pool::Pool,
    types::{ datapoint::SymbolData, weighted_symbol::WeightedSymbol },
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
        weighted_symbols: &Vec<WeightedSymbol>,
        interval: time::Duration
    ) -> Result<()> {
        loop {
            self.rebalance(weighted_symbols).await?;
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn rebalance(&self, weighted_symbols: &Vec<WeightedSymbol>) -> Result<()> {
        let fetcher = SanctumHistoricalApyFetcher::new();
        let mut symbol_datas = vec![];

        for weighted_symbol in weighted_symbols {
            let datapoints = fetcher.fetch(&weighted_symbol.symbol).await?;
            symbol_datas.push(SymbolData {
                symbol: weighted_symbol.symbol.clone(),
                datapoints,
            });
        }

        let allocator = EmaAllocator::new(Some(2), Some(5));
        let mut allocations = allocator.allocate(symbol_datas)?;
        allocations.validate()?;

        allocations.apply_weights(weighted_symbols);
        allocations.validate()?;

        let pool = MaxPool::new("");
        let current_pool_allocations = pool.get_allocation()?;
        let pool_allocation_changes = pool.get_allocation_changes(
            &current_pool_allocations,
            &allocations
        )?;

        info!("pool allocations: {:?}", current_pool_allocations);
        info!("pool allocation changes: {:?}", pool_allocation_changes);

        Ok(())
    }
}

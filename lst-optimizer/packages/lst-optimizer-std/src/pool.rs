use anyhow::Result;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::{
    allocator::AllocationRatios,
    types::{
        context::Context,
        pool_allocation::PoolAllocations,
        pool_allocation_changes::{
            PoolAllocationChanges, PoolAllocationLamportsChanges, PoolAssetChange,
        },
    },
};

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("Failed to fetch allocations")]
    FailedToFetchAllocations,

    #[error(
        "Failed to calculate lamports per symbol, the lamports are {0}, the symbol bps is {1}"
    )]
    FailedToCalculateLamportsPerSymbol(u64, Decimal),

    #[error("Failed to calculate allocation changes")]
    FailedToCalculateAllocationChanges,
}

#[async_trait::async_trait]
pub trait PoolAllocable {
    async fn get_allocation(&self, context: &Context) -> Result<PoolAllocations>;
    async fn get_allocation_lamports_changes(
        &self,
        context: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios,
    ) -> Result<PoolAllocationLamportsChanges>;
    async fn get_allocation_changes(
        &self,
        context: &Context,
        pool_allocations: &PoolAllocations,
        new_allocation_ratios: &AllocationRatios,
    ) -> Result<PoolAllocationChanges>;
}

#[async_trait::async_trait]
pub trait PoolRebalancable {
    async fn rebalance_asset(
        &self,
        context: &Context,
        pool_asset_change: &PoolAssetChange,
    ) -> Result<()>;
}

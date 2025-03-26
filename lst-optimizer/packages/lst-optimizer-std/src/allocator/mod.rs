use anyhow::Result;
use lst_optimizer_utils::logger::info;
use rust_decimal::{prelude::Zero, Decimal};

use crate::types::{asset::Asset, datapoint::SymbolData, pool_allocation::MAX_ALLOCATION_BPS};

pub trait Allocator<T> {
    fn allocate(&self, symbol_datas: Vec<SymbolData<T>>) -> Result<AllocationRatios>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllocationRatio {
    pub bps: Decimal,
    pub mint: String,
}

impl AllocationRatio {
    pub fn new(mint: &str, bps: i16) -> Self {
        Self {
            mint: mint.to_string(),
            bps: Decimal::from(bps),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllocationRatios {
    pub asset_alloc_ratios: Vec<AllocationRatio>,
}

impl AllocationRatios {
    pub fn new(asset_alloc_ratios: Vec<AllocationRatio>) -> Self {
        Self { asset_alloc_ratios }
    }

    pub fn apply_weights(&mut self, assets: &Vec<Asset>) {
        let max_allocation_bps = Decimal::from(MAX_ALLOCATION_BPS);

        // Filter out weights that are not in the allocation ratios
        let assets: &Vec<Asset> = &assets
            .iter()
            .filter(|asset| {
                let s = &asset.mint.to_lowercase();
                self.asset_alloc_ratios.iter().any(|symbol_ratio| {
                    let sr = &symbol_ratio.mint.to_lowercase();
                    sr.eq(s)
                })
            })
            .cloned()
            .collect();

        // Calculate total weight of all assets
        let mut total_weight = Decimal::zero();
        for asset in assets {
            total_weight += asset.weight;
        }

        // Adjust allocation ratios based on weights
        for symbol_ratio in self.asset_alloc_ratios.iter_mut() {
            for asset in assets.iter() {
                if symbol_ratio.mint == asset.mint {
                    symbol_ratio.bps = (asset.weight / total_weight) * max_allocation_bps;
                    info!(
                        "Adjusted weight for {} to {} = {}",
                        symbol_ratio.mint, asset.weight, symbol_ratio.bps
                    );
                }
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        let mut total_allocation = Decimal::zero();
        for symbol_ratio in self.asset_alloc_ratios.iter() {
            total_allocation += symbol_ratio.bps;
        }

        let max_allcation_bps = Decimal::from(MAX_ALLOCATION_BPS);
        if total_allocation != max_allcation_bps {
            return Err(anyhow::anyhow!(format!(
                "Total allocation ({}) bps is not equal to max allocation {}",
                total_allocation, max_allcation_bps
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_apply_weight_success() {
        let assets = vec![
            Asset::new_with_weight("jupsol", 0.8),
            Asset::new_with_weight("inf", 0.2),
        ];
        let mut allocation = AllocationRatios::new(vec![
            AllocationRatio {
                bps: (5000).into(),
                mint: "jupsol".to_string(),
            },
            AllocationRatio {
                bps: (5000).into(),
                mint: "inf".to_string(),
            },
        ]);
        allocation.apply_weights(&assets);
        assert_eq!(allocation.asset_alloc_ratios[0].bps, (8000).into());
        assert_eq!(allocation.asset_alloc_ratios[1].bps, (2000).into());
    }

    #[test]
    fn test_allocation_apply_weight_success_edge() {
        let assets = vec![Asset::new_with_weight("jupsol", 0.1)];
        let mut allocation = AllocationRatios::new(vec![AllocationRatio {
            bps: (10000).into(),
            mint: "jupsol".to_string(),
        }]);
        allocation.apply_weights(&assets);
        assert_eq!(allocation.asset_alloc_ratios[0].bps, (10000).into());
    }

    #[test]
    fn test_allocation_apply_weight_success_edge_unrelated_weights() {
        // Should apply only allocated asset's weight
        let assets = vec![
            Asset::new_with_weight("jupsol", 0.1),
            Asset::new_with_weight("jitosol", 0.9), // This should be ignored
        ];
        let mut allocation = AllocationRatios::new(vec![AllocationRatio {
            bps: (10000).into(),
            mint: "jupsol".to_string(),
        }]);
        allocation.apply_weights(&assets);
        assert_eq!(allocation.asset_alloc_ratios[0].bps, (10000).into());
    }

    #[test]
    fn test_allocation_total_ratio_validation_succcess() {
        let allocation = AllocationRatios::new(vec![
            AllocationRatio {
                bps: (5000).into(),
                mint: "jupsol".to_string(),
            },
            AllocationRatio {
                bps: (5000).into(),
                mint: "inf".to_string(),
            },
        ]);
        assert!(allocation.validate().is_ok());
    }

    #[test]
    fn test_allocation_total_ratio_validation_fail() {
        let allocation = AllocationRatios::new(vec![
            AllocationRatio {
                bps: (1000).into(),
                mint: "jupsol".to_string(),
            },
            AllocationRatio {
                bps: (1000).into(),
                mint: "inf".to_string(),
            },
        ]);
        assert!(allocation.validate().is_err());
    }
}

use anyhow::Result;
use rust_decimal::{ prelude::Zero, Decimal };

use crate::types::{
    pool_allocation::MAX_ALLOCATION_BPS,
    datapoint::SymbolData,
    weighted_symbol::WeightedSymbol,
};

pub trait Allocator<T> {
    fn allocate(&self, symbol_datas: Vec<SymbolData<T>>) -> Result<AllocationRatios>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllocationRatio {
    pub bps: Decimal,
    pub symbol: String,
}

impl AllocationRatio {
    pub fn new(symbol: &str, bps: i128) -> Self {
        Self {
            bps: Decimal::from(bps),
            symbol: symbol.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllocationRatios {
    pub symbol_ratios: Vec<AllocationRatio>,
}

impl AllocationRatios {
    pub fn new(symbol_ratios: Vec<AllocationRatio>) -> Self {
        Self { symbol_ratios }
    }

    pub fn apply_weights(&mut self, weights: &Vec<WeightedSymbol>) {
        let max_allocation_bps = Decimal::from(MAX_ALLOCATION_BPS);

        // Filter out weights that are not in the allocation ratios
        let weights: &Vec<WeightedSymbol> = &weights
            .iter()
            .filter(|weighted_symbol| {
                let ws = &weighted_symbol.symbol.to_lowercase();
                self.symbol_ratios.iter().any(|symbol_ratio| {
                    let sr = &symbol_ratio.symbol.to_lowercase();
                    sr.eq(ws)
                })
            })
            .cloned()
            .collect();

        // Calculate total weight of all weighted symbols
        let mut total_weight = Decimal::zero();
        for weighted_symbol in weights {
            total_weight += weighted_symbol.weight;
        }

        // Adjust allocation ratios based on weights
        for symbol_ratio in self.symbol_ratios.iter_mut() {
            for weights in weights.iter() {
                if symbol_ratio.symbol == weights.symbol {
                    symbol_ratio.bps = (weights.weight / total_weight) * max_allocation_bps;
                }
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        let mut total_allocation = Decimal::zero();
        for symbol_ratio in self.symbol_ratios.iter() {
            total_allocation += symbol_ratio.bps;
        }

        let max_allcation_bps = Decimal::from(MAX_ALLOCATION_BPS);
        if total_allocation != max_allcation_bps {
            return Err(
                anyhow::anyhow!(
                    format!(
                        "Total allocation ({}) bps is not equal to max allocation {}",
                        total_allocation,
                        max_allcation_bps
                    )
                )
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_apply_weight_success() {
        let weighted_symbols = vec![
            WeightedSymbol::new("jupsol", 0.8),
            WeightedSymbol::new("inf", 0.2)
        ];
        let mut allocation = AllocationRatios::new(
            vec![
                AllocationRatio {
                    bps: (5000).into(),
                    symbol: "jupsol".to_string(),
                },
                AllocationRatio {
                    bps: (5000).into(),
                    symbol: "inf".to_string(),
                }
            ]
        );
        allocation.apply_weights(&weighted_symbols);
        assert_eq!(allocation.symbol_ratios[0].bps, (8000).into());
        assert_eq!(allocation.symbol_ratios[1].bps, (2000).into());
    }

    #[test]
    fn test_allocation_apply_weight_success_edge() {
        let weighted_symbols = vec![WeightedSymbol::new("jupsol", 0.1)];
        let mut allocation = AllocationRatios::new(
            vec![AllocationRatio {
                bps: (10000).into(),
                symbol: "jupsol".to_string(),
            }]
        );
        allocation.apply_weights(&weighted_symbols);
        assert_eq!(allocation.symbol_ratios[0].bps, (10000).into());
    }

    #[test]
    fn test_allocation_apply_weight_success_edge_unrelated_weights() {
        // Should apply only allocated asset's weight
        let weighted_symbols = vec![
            WeightedSymbol::new("jupsol", 0.1),
            WeightedSymbol::new("jitosol", 0.9) // This should be ignored
        ];
        let mut allocation = AllocationRatios::new(
            vec![AllocationRatio {
                bps: (10000).into(),
                symbol: "jupsol".to_string(),
            }]
        );
        allocation.apply_weights(&weighted_symbols);
        assert_eq!(allocation.symbol_ratios[0].bps, (10000).into());
    }

    #[test]
    fn test_allocation_total_ratio_validation_succcess() {
        let allocation = AllocationRatios::new(
            vec![
                AllocationRatio {
                    bps: (5000).into(),
                    symbol: "jupsol".to_string(),
                },
                AllocationRatio {
                    bps: (5000).into(),
                    symbol: "inf".to_string(),
                }
            ]
        );
        assert!(allocation.validate().is_ok());
    }

    #[test]
    fn test_allocation_total_ratio_validation_fail() {
        let allocation = AllocationRatios::new(
            vec![
                AllocationRatio {
                    bps: (1000).into(),
                    symbol: "jupsol".to_string(),
                },
                AllocationRatio {
                    bps: (1000).into(),
                    symbol: "inf".to_string(),
                }
            ]
        );
        assert!(allocation.validate().is_err());
    }
}

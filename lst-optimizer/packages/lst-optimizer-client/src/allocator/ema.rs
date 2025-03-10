use anyhow::{ Context, Result };
use log::debug;
use lst_optimizer_std::{
    allocator::{ AllocationRatio, AllocationRatios, Allocator },
    fetcher::apy::Apy,
    types::{ pool_allocation::MAX_ALLOCATION_BPS, datapoint::SymbolData },
};
use rust_decimal::Decimal;
use ta::{ indicators::ExponentialMovingAverage, Next };
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Ema {
    pub mint: String,
    pub ema: f64,
}

#[derive(Debug, Error)]
pub enum EmaError {
    #[error("Failed to calculate EMA")]
    FailedToCalculateEma,

    #[error("Failed to divide max allocation bps")]
    FailedToDivideMaxAllocationBps,
}

#[derive(Debug, Clone)]
pub struct EmaAllocator {
    allocation_limit: Option<usize>,
    period: Option<usize>,
}

impl EmaAllocator {
    pub fn new(allocation_limit: Option<usize>, period: Option<usize>) -> Self {
        Self { allocation_limit, period }
    }

    fn calculate_emas(&self, datapoints: &Vec<Apy>, period: usize) -> Result<Vec<f64>> {
        let mut ema = ExponentialMovingAverage::new(period).context(
            EmaError::FailedToCalculateEma
        )?;
        let mut emas: Vec<f64> = Vec::new();
        for datapoint in datapoints {
            let value = ema.next(datapoint.apy);
            emas.push(value);
        }
        Ok(emas)
    }

    fn sort_emas_desc(&self, emas: Vec<Ema>) -> Vec<Ema> {
        let mut emas = emas;
        emas.sort_by(|a, b| b.ema.partial_cmp(&a.ema).unwrap());
        emas
    }

    fn truncate_emas(&self, emas: Vec<Ema>, limit: usize) -> Vec<Ema> {
        emas.into_iter().take(limit).collect()
    }

    fn allocate_equal(&self, emas: Vec<Ema>) -> Result<Vec<AllocationRatio>> {
        let mut ratios: Vec<AllocationRatio> = vec![];
        let number_of_symbol = Decimal::from(emas.len());
        let max_allocation_bps = Decimal::from(MAX_ALLOCATION_BPS);
        let bps_per_symbol = max_allocation_bps
            .checked_div(number_of_symbol)
            .context(EmaError::FailedToDivideMaxAllocationBps)?;

        for ema in emas {
            let ratio = AllocationRatio {
                bps: bps_per_symbol,
                mint: ema.mint,
            };
            ratios.push(ratio);
        }
        Ok(ratios)
    }
}

impl Allocator<Apy> for EmaAllocator {
    fn allocate(&self, symbol_datas: Vec<SymbolData<Apy>>) -> Result<AllocationRatios> {
        let mut latest_emas: Vec<Ema> = Vec::new();
        for symbol_data in symbol_datas {
            let datapoints = &symbol_data.datapoints;
            let emas = self.calculate_emas(datapoints, self.period.unwrap_or(5))?;
            latest_emas.push(Ema {
                mint: symbol_data.mint.clone(),
                ema: emas.last().unwrap().to_owned(),
            });
        }
        let mut emas = self.sort_emas_desc(latest_emas);
        debug!("Sorted EMAs: {:?}", emas);
        if self.allocation_limit.is_some() {
            emas = self.truncate_emas(emas, self.allocation_limit.unwrap());
        }
        let ratios = self.allocate_equal(emas)?;
        Ok(AllocationRatios::new(ratios))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_datapoints_asc() -> Vec<Apy> {
        vec![
            Apy {
                mint: "".to_string(),
                apy: 1.0,
            },
            Apy {
                mint: "".to_string(),
                apy: 2.0,
            },
            Apy {
                mint: "".to_string(),
                apy: 3.0,
            },
            Apy {
                mint: "".to_string(),
                apy: 4.0,
            },
            Apy {
                mint: "".to_string(),
                apy: 5.0,
            }
        ]
    }

    fn test_emas() -> Vec<Ema> {
        vec![
            Ema {
                mint: "".to_string(),
                ema: 1.0,
            },
            Ema {
                mint: "".to_string(),
                ema: 2.0,
            },
            Ema {
                mint: "".to_string(),
                ema: 3.0,
            },
            Ema {
                mint: "".to_string(),
                ema: 4.0,
            },
            Ema {
                mint: "".to_string(),
                ema: 5.0,
            }
        ]
    }

    #[test]
    fn test_calculate_emas_success() {
        let datapoints = test_datapoints_asc();
        let expected_emas = vec![
            1.0,
            1.6666666666666665,
            2.5555555555555554,
            3.518518518518518,
            4.506172839506172
        ];
        let ema_values = EmaAllocator::new(None, None).calculate_emas(&datapoints, 2);
        assert_eq!(ema_values.unwrap(), expected_emas);
    }

    #[test]
    fn test_calculate_emas_success_on_empty() {
        let ema_values = EmaAllocator::new(None, None).calculate_emas(&vec![], 2);
        assert_eq!(ema_values.unwrap(), Vec::<f64>::new());
    }

    #[test]
    fn test_calculate_emas_fail_by_zero_period() {
        let datapoints = test_datapoints_asc();
        let ema_values = EmaAllocator::new(None, None).calculate_emas(&datapoints, 0);
        assert_eq!(
            ema_values.err().unwrap().to_string(),
            EmaError::FailedToCalculateEma.to_string()
        );
    }

    #[test]
    fn test_sort_ema() {
        let emas = test_emas();
        let sorted_emas = EmaAllocator::new(None, None).sort_emas_desc(emas);
        assert_eq!(
            sorted_emas,
            vec![
                Ema {
                    mint: "".to_string(),
                    ema: 5.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 4.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 3.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 2.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 1.0,
                }
            ]
        );
    }

    #[test]
    fn test_truncate_ema() {
        let emas = test_emas();
        let truncated_emas = EmaAllocator::new(None, None).truncate_emas(emas, 3);
        assert_eq!(
            truncated_emas,
            vec![
                Ema {
                    mint: "".to_string(),
                    ema: 1.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 2.0,
                },
                Ema {
                    mint: "".to_string(),
                    ema: 3.0,
                }
            ]
        );
    }

    #[test]
    fn test_allocate_equal_success() {
        let emas = test_emas();
        let ratios = EmaAllocator::new(None, None).allocate_equal(emas).unwrap();
        assert_eq!(
            ratios,
            vec![
                AllocationRatio {
                    bps: Decimal::from(2000),
                    mint: "".to_string(),
                },
                AllocationRatio {
                    bps: Decimal::from(2000),
                    mint: "".to_string(),
                },
                AllocationRatio {
                    bps: Decimal::from(2000),
                    mint: "".to_string(),
                },
                AllocationRatio {
                    bps: Decimal::from(2000),
                    mint: "".to_string(),
                },
                AllocationRatio {
                    bps: Decimal::from(2000),
                    mint: "".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_allocate_equal_fail_on_empty() {
        let emas = Vec::<Ema>::new();
        let ratios = EmaAllocator::new(None, None).allocate_equal(emas);
        assert_eq!(
            ratios.err().unwrap().to_string(),
            EmaError::FailedToDivideMaxAllocationBps.to_string()
        );
    }
}

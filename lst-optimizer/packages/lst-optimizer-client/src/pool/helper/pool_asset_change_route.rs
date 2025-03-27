use anyhow::Result;
use controller_lib::{calculator::typedefs::CalculatorType, Pubkey};
use lst_optimizer_std::types::{
    amount_change::AmountChange, asset::Asset, pool_allocation_changes::PoolAssetChange,
};
use spl_token::native_mint;

use crate::typedefs::pool_to_calculator_type;

#[derive(Debug, Clone)]
pub struct PoolAssetChangeRoute {
    pub src_mint: Pubkey,
    pub dst_mint: Pubkey,
    pub src_cal: CalculatorType,
    pub dst_cal: CalculatorType,
    pub amount: u64,
}

pub trait PoolAssetChangeRouter {
    fn get_route(&self, asset: &Asset) -> Result<PoolAssetChangeRoute>;
}

impl PoolAssetChangeRouter for PoolAssetChange {
    fn get_route(&self, asset: &Asset) -> Result<PoolAssetChangeRoute> {
        let ret = match self.amount {
            AmountChange::Increase {
                lamports,
                lst_amount: _,
            } => (
                native_mint::ID,
                self.mint.parse()?,
                CalculatorType::Wsol,
                pool_to_calculator_type(asset)?,
                lamports, // use lamports for amount
            ),
            AmountChange::Decrease {
                lamports: _,
                lst_amount,
            } => (
                self.mint.parse()?,
                native_mint::ID,
                pool_to_calculator_type(asset)?,
                CalculatorType::Wsol,
                lst_amount,
            ),
        };
        Ok(PoolAssetChangeRoute {
            src_mint: ret.0,
            dst_mint: ret.1,
            src_cal: ret.2,
            dst_cal: ret.3,
            amount: ret.4,
        })
    }
}

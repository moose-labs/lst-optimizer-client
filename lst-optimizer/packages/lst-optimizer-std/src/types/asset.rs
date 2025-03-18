use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub mint: String,
    pub symbol: String,
    pub weight: Decimal,
    pub token_program: String,
    pub pool: Option<PoolInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PoolInfo {
    pub program: String,
    pub pool: Option<String>,
    pub validator_list: Option<String>,
    pub vote_account: Option<String>,
    pub program_id: Option<String>,
}

impl Asset {
    pub fn new(mint: &str, symbol: &str, weight: f64) -> Self {
        Self {
            mint: mint.to_string(),
            symbol: symbol.to_string(),
            weight: Decimal::from_f64(weight).unwrap(),
            token_program: "".to_string(),
            pool: None,
        }
    }

    pub fn new_with_weight(mint: &str, weight: f64) -> Self {
        Asset::new(mint, "", weight)
    }

    pub fn new_with_symbol(mint: &str, symbol: &str) -> Self {
        Asset::new(mint, symbol, 1.0)
    }
}

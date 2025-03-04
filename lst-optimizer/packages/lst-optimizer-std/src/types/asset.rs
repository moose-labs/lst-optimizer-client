use rust_decimal::{ prelude::FromPrimitive, Decimal };

#[derive(Debug, Clone)]
pub struct Asset {
    pub symbol: String,
    pub weight: Decimal,
}

impl Asset {
    pub fn new(symbol: &str) -> Self {
        Asset::new_with_weight(symbol, 1.0)
    }

    pub fn new_with_weight(symbol: &str, weight: f64) -> Self {
        Self { symbol: symbol.to_string(), weight: Decimal::from_f64(weight).unwrap() }
    }
}

use rust_decimal::{ prelude::FromPrimitive, Decimal };

#[derive(Debug, Clone)]
pub struct WeightedSymbol {
    pub symbol: String,
    pub weight: Decimal,
}

impl WeightedSymbol {
    pub fn new(symbol: &str, weight: f64) -> Self {
        Self { symbol: symbol.to_string(), weight: Decimal::from_f64(weight).unwrap() }
    }
}

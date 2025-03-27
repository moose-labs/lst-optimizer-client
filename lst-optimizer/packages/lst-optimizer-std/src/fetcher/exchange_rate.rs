use rust_decimal::Decimal;

use crate::types::datapoint::Datapoint;

// ExchangeRate is a datapoint that represents the exchange rate of liquid staking token to the underlying asset

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    pub symbol: String,
    pub rate: Decimal,
}

impl Datapoint for ExchangeRate {
    fn get_symbol(&self) -> String {
        self.symbol.to_owned()
    }
}

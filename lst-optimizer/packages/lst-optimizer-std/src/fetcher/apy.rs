use crate::types::datapoint::Datapoint;

// APY is a datapoint that represents the APY of a liquid staking pool

#[derive(Debug, Clone)]
pub struct Apy {
    pub symbol: String,
    pub apy: f64,
}

impl Datapoint for Apy {
    fn get_symbol(&self) -> String {
        self.symbol.to_owned()
    }
}

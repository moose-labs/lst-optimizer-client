// APY is a datapoint that represents the APY of a liquid staking pool

#[derive(Debug, Clone)]
pub struct Apy {
    pub mint: String,
    pub apy: f64,
}

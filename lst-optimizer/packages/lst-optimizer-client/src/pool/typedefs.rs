#[derive(Debug, Clone)]
pub struct MaxPoolOptions {
    pub rpc_url: String,
    pub minimum_rebalance_lamports: u64,
}

impl Default for MaxPoolOptions {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            minimum_rebalance_lamports: 1_000_000,
        }
    }
}

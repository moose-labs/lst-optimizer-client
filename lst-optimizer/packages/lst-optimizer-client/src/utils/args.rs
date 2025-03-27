use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "optimizer")]
pub struct AppArgs {
    /// Rebalancing authority keypair file
    #[arg(
        long,
        default_value = "/Users/imalice/.config/solana/mAXReBWzSH7EcsX7ZEvcqmgU2n4TZ44jgsyAud54oYq.json"
    )]
    pub keypair: String,

    /// Rpc url
    /// (default: "https://api.mainnet-beta.solana.com")
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub url: String,

    /// Rebalancing interval in seconds
    /// (default: 172800 seconds (2 days))
    #[arg(long, default_value_t = 172800)]
    pub interval: u64,

    /// Minimum lamports to rebalance
    /// (default: 1_000_000_000)
    #[arg(long, short, default_value_t = 1_000_000_000)]
    pub minimum_rebalance_lamports: u64,
}

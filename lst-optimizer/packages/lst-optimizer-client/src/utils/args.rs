use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "optimizer")]
pub struct AppArgs {
    /// Rebalancing authority keypair file
    #[arg(long)]
    pub keypair: String,

    /// Rpc url
    /// (default: "https://api.mainnet-beta.solana.com")
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub url: String,

    /// Rebalancing interval in seconds
    /// (default: 172800 seconds (2 days))
    #[arg(long, default_value = "172800")]
    pub interval: u64,
}

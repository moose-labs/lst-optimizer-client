#[derive(Debug, Clone)]
pub struct PoolAsset {
    pub mint: String,
    pub lamports: i128,
    pub reserves: u64,
}

impl PoolAsset {
    pub fn new(mint: &str, lamports: i128, reserves: u64) -> Self {
        Self { mint: mint.to_string(), lamports, reserves }
    }
}

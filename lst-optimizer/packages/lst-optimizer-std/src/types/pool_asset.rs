#[derive(Debug, Clone)]
pub struct PoolAsset {
    pub symbol: String,
    pub lamports: i128,
}

impl PoolAsset {
    pub fn new(symbol: &str, lamports: i128) -> Self {
        Self { symbol: symbol.to_string(), lamports }
    }
}

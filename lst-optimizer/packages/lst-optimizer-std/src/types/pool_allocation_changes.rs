// Change in lamports
#[derive(Debug, Clone)]
pub struct PoolAllocationLamportsChanges {
    pub assets: Vec<PoolAssetLamportsChange>,
}

impl PoolAllocationLamportsChanges {
    pub fn new(assets: Vec<PoolAssetLamportsChange>) -> Self {
        Self { assets }
    }

    pub fn get_asset_lamports_changes(&self, mint: &str) -> Option<&PoolAssetLamportsChange> {
        for asset in self.assets.iter() {
            if asset.mint.eq(&mint) {
                return Some(asset);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct PoolAssetLamportsChange {
    pub mint: String,
    pub lamports: i128,
}

impl PoolAssetLamportsChange {
    pub fn new(mint: &str, lamports: i128) -> Self {
        Self { mint: mint.to_string(), lamports }
    }
}

// Change in lst amount
#[derive(Debug, Clone)]
pub struct PoolAllocationChanges {
    pub assets: Vec<PoolAssetChange>,
}

impl PoolAllocationChanges {
    pub fn new(assets: Vec<PoolAssetChange>) -> Self {
        Self { assets }
    }

    pub fn get_asset_changes(&self, mint: &str) -> Option<&PoolAssetChange> {
        for asset in self.assets.iter() {
            if asset.mint.eq(&mint) {
                return Some(asset);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct PoolAssetChange {
    pub mint: String,
    pub amount: i128,
}

impl PoolAssetChange {
    pub fn new(mint: &str, amount: i128) -> Self {
        Self { mint: mint.to_string(), amount }
    }
}

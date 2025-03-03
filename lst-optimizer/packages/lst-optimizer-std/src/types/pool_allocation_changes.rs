use super::pool_asset::PoolAsset;

#[derive(Debug, Clone)]
pub struct PoolAllocationChanges {
    pub assets: Vec<PoolAsset>,
}

impl PoolAllocationChanges {
    pub fn new(assets: Vec<PoolAsset>) -> Self {
        Self { assets }
    }

    pub fn get_asset_changes(&self, symbol: &str) -> Option<&PoolAsset> {
        for asset in self.assets.iter() {
            if asset.symbol.eq(&symbol) {
                return Some(asset);
            }
        }
        None
    }
}

use super::pool_asset::PoolAsset;

pub const MAX_ALLOCATION_BPS: i128 = 10_000;

#[derive(Debug, Clone)]
pub struct PoolAllocations {
    pub assets: Vec<PoolAsset>,
}

impl PoolAllocations {
    pub fn get_total_lamports(&self) -> i128 {
        let mut total = 0;
        for asset in self.assets.iter() {
            total += asset.lamports;
        }
        total
    }

    pub fn get_pool_asset(&self, symbol: &str) -> Option<&PoolAsset> {
        for asset in self.assets.iter() {
            if asset.symbol.eq(&symbol) {
                return Some(asset);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pool_asset() {
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("jupsol", 100),
                PoolAsset::new("inf", 200),
                PoolAsset::new("jitosol", 300),
                PoolAsset::new("hsol", 400)
            ],
        };

        let asset = pool_allocations.get_pool_asset("inf");
        assert_eq!(asset.is_some(), true);
        assert_eq!(asset.unwrap().lamports, 200);

        let asset = pool_allocations.get_pool_asset("sol");
        assert_eq!(asset.is_none(), true);
    }
}

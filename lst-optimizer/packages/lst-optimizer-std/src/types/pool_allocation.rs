use super::{ asset::Asset, pool_asset::PoolAsset };
use anyhow::Result;

pub const MAX_ALLOCATION_BPS: i16 = 10_000;

#[derive(Debug, Clone)]
pub struct PoolAllocations {
    pub assets: Vec<PoolAsset>,
}

impl PoolAllocations {
    pub fn get_total_lamports(&self) -> u64 {
        let mut total = 0;
        for asset in self.assets.iter() {
            total += asset.lamports;
        }
        total
    }

    pub fn get_pool_asset(&self, mint: &str) -> Option<&PoolAsset> {
        for asset in self.assets.iter() {
            if asset.mint.eq(&mint) {
                return Some(asset);
            }
        }
        None
    }

    pub fn assert_pool_allocations_are_defined(&self, assets: &Vec<Asset>) -> Result<()> {
        for asset in assets.iter() {
            let mut is_defined = false;
            for pool_asset in self.assets.iter() {
                if pool_asset.mint.eq(&asset.mint) {
                    is_defined = true;
                }
            }
            if is_defined == false {
                return Err(
                    anyhow::anyhow!("Pool asset {} is not defined in the asset list", asset.mint)
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pool_asset() {
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("jupsol", 100, 0),
                PoolAsset::new("inf", 200, 0),
                PoolAsset::new("jitosol", 300, 0),
                PoolAsset::new("hsol", 400, 0)
            ],
        };

        let asset = pool_allocations.get_pool_asset("inf");
        assert_eq!(asset.is_some(), true);
        assert_eq!(asset.unwrap().lamports, 200);

        let asset = pool_allocations.get_pool_asset("sol");
        assert_eq!(asset.is_none(), true);
    }

    #[test]
    fn test_validate_pool_allocations_are_defined() {
        let pool_allocations = PoolAllocations {
            assets: vec![
                PoolAsset::new("jupsol", 100, 0),
                PoolAsset::new("inf", 200, 0),
                PoolAsset::new("jitosol", 300, 0),
                PoolAsset::new("hsol", 400, 0)
            ],
        };

        let defined_assets = vec![Asset::new("jupsol", "", 0.5), Asset::new("inf", "", 0.5)];
        let result = pool_allocations.assert_pool_allocations_are_defined(&defined_assets);
        assert_eq!(result.is_ok(), true);

        let defined_assets = vec![Asset::new("jitosol", "", 0.5), Asset::new("sol", "", 0.5)];
        let result = pool_allocations.assert_pool_allocations_are_defined(&defined_assets);
        assert_eq!(result.is_err(), true);

        let defined_assets = vec![
            Asset::new("jupsol", "", 0.5),
            Asset::new("inf", "", 0.5),
            Asset::new("jitosol", "", 0.5),
            Asset::new("hsol", "", 0.5)
        ];
        let result = pool_allocations.assert_pool_allocations_are_defined(&defined_assets);
        assert_eq!(result.is_err(), false);
    }
}

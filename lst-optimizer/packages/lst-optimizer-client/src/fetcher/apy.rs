use std::{collections::HashMap, thread::sleep};

use anyhow::Result;
use lst_optimizer_std::{
    fetcher::{apy::Apy, fetcher::Fetcher},
    types::asset::Asset,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SanctumHistoricalResponse {
    apys: HashMap<String, Vec<SanctumEpochApy>>,
}

#[derive(Debug, Deserialize)]
struct SanctumEpochApy {
    apy: f64,
}

pub struct SanctumHistoricalApyFetcher {}

impl SanctumHistoricalApyFetcher {
    pub fn new() -> Self {
        Self {}
    }

    fn get_symbol_endpoint(&self, symbol: &String) -> String {
        format!(
            "https://extra-api.sanctum.so/v1/apy/indiv-epochs?lst={}&n=300",
            symbol
        )
    }

    fn is_exceptable_symbol(&self, symbol: &String) -> bool {
        symbol.to_lowercase().eq("sol")
    }
}

#[async_trait::async_trait]
impl Fetcher<Apy> for SanctumHistoricalApyFetcher {
    async fn fetch(&self, asset: &Asset) -> Result<Vec<Apy>> {
        if self.is_exceptable_symbol(&asset.symbol) {
            return Ok(vec![Apy {
                mint: asset.mint.clone(),
                apy: 0.0,
            }]);
        }

        let client = reqwest::Client::new();
        let url = self.get_symbol_endpoint(&asset.symbol);
        let response: SanctumHistoricalResponse = client.get(url).send().await?.json().await?;

        let mut datapoints: Vec<Apy> = vec![];
        response.apys.iter().for_each(|(_, apys)| {
            apys.iter().for_each(|apy| {
                datapoints.push(Apy {
                    mint: asset.mint.clone(),
                    apy: apy.apy,
                });
            });
        });

        sleep(std::time::Duration::from_millis(200));

        if datapoints.is_empty() {
            return Err(anyhow::anyhow!("No datapoints found for {}", asset.symbol));
        }

        Ok(datapoints)
    }
}

#[cfg(test)]
mod tests {
    use lst_optimizer_std::types::asset::Asset;

    use super::*;

    #[tokio::test]
    async fn test_fetch() {
        let fetcher = SanctumHistoricalApyFetcher::new();
        let datapoints = fetcher.fetch(&Asset::new("", "inf", 1.0)).await.unwrap();
        assert_ne!(datapoints.len(), 0);
    }
}

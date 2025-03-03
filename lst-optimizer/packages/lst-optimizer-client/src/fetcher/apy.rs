use std::{ collections::HashMap, thread::sleep };

use anyhow::Result;
use log::info;
use lst_optimizer_std::fetcher::{ apy::Apy, fetcher::Fetcher };
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
}

#[async_trait::async_trait]
impl Fetcher<Apy> for SanctumHistoricalApyFetcher {
    async fn fetch(&self, symbol: &str) -> Result<Vec<Apy>> {
        info!("fetching historical APY data for {}", symbol);

        let client = reqwest::Client::new();
        let url = format!("https://extra-api.sanctum.so/v1/apy/indiv-epochs?lst={}&n=300", symbol);
        let response: SanctumHistoricalResponse = client.get(url).send().await?.json().await?;

        let mut datapoints: Vec<Apy> = vec![];
        response.apys.iter().for_each(|(symbol, apys)| {
            apys.iter().for_each(|apy| {
                datapoints.push(Apy {
                    symbol: symbol.clone(),
                    apy: apy.apy,
                });
            });
        });

        sleep(std::time::Duration::from_millis(200));

        if datapoints.is_empty() {
            return Err(anyhow::anyhow!("No datapoints found for {}", symbol));
        }

        Ok(datapoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch() {
        let fetcher = SanctumHistoricalApyFetcher::new();
        let datapoints = fetcher.fetch("inf").await.unwrap();
        assert_ne!(datapoints.len(), 0);
    }
}

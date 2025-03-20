use anyhow::Result;

use crate::types::asset::Asset;

#[async_trait::async_trait]
pub trait Fetcher<T> {
    async fn fetch(&self, asset: &Asset) -> Result<Vec<T>>;
}

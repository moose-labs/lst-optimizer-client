use anyhow::Result;

#[async_trait::async_trait]
pub trait Fetcher<T> {
    async fn fetch(&self, symbol: &str) -> Result<Vec<T>>;
}

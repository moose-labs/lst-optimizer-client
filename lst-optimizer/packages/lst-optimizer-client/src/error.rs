use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AppError {
    #[error("Failed to retry rebalance pool asset change")]
    FailedToRetryRebalancePoolAssetChange,
}

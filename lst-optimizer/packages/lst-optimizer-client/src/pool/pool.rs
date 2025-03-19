use super::typedefs::MaxPoolOptions;
use anyhow::{Context as _AnyhowContext, Ok, Result};
use controller_lib::controller::ControllerClient;
use controller_lib::Pubkey;
use lst_optimizer_std::{pool::PoolError, types::pool_allocation::MAX_ALLOCATION_BPS};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;

pub struct MaxPool {
    // A controller program id
    program_id: Pubkey,
    options: MaxPoolOptions,
    controller_client: ControllerClient,
}

impl MaxPool {
    pub fn new(program_id: &str, options: MaxPoolOptions) -> Self {
        let rpc_client = RpcClient::new(options.rpc_url.clone());
        Self {
            program_id: program_id.parse().unwrap(),
            options: options,
            controller_client: ControllerClient::new(rpc_client),
        }
    }

    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    pub fn controller_client(&self) -> &ControllerClient {
        &self.controller_client
    }

    pub fn pool_options(&self) -> &MaxPoolOptions {
        &self.options
    }

    pub fn calculate_lamports_from_bps(
        &self,
        total_lamports: u64,
        symbol_bps: Decimal,
    ) -> Result<u64> {
        let total_lamports_dec = Decimal::from(total_lamports);
        let max_bps = Decimal::from(MAX_ALLOCATION_BPS);
        let ratio = symbol_bps.checked_div(max_bps).context(
            PoolError::FailedToCalculateLamportsPerSymbol(total_lamports, symbol_bps),
        )?;

        let target_lamports = ratio
            .checked_mul(total_lamports_dec)
            .context(PoolError::FailedToCalculateAllocationChanges)?;

        Ok(target_lamports
            .ceil()
            .to_u64()
            .ok_or(PoolError::FailedToCalculateLamportsPerSymbol(
                total_lamports,
                symbol_bps,
            ))?)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_calculate_lamports_per_symbol_success() {
        let pool = MaxPool::new("", MaxPoolOptions::default());
        let total_lamports = 1_000_000;
        let symbol_bps = Decimal::from(5000);
        let target_lamports = pool
            .calculate_lamports_from_bps(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 500_000);
    }

    #[tokio::test]
    async fn test_calculate_lamports_per_symbol_success_on_division_by_zeros() {
        let pool = MaxPool::new("", MaxPoolOptions::default());

        // symbol_bps = 0
        let total_lamports = 1_000_000;
        let symbol_bps = Decimal::from(0);
        let target_lamports = pool
            .calculate_lamports_from_bps(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 0);

        // total_lamports = 0
        let total_lamports = 0;
        let symbol_bps = Decimal::from(1000);
        let target_lamports = pool
            .calculate_lamports_from_bps(total_lamports, symbol_bps)
            .unwrap();
        assert_eq!(target_lamports, 0);
    }
}

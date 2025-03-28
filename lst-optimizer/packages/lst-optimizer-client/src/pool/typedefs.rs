#[derive(Debug, Clone)]
pub struct MaxPoolOptions {
    pub rpc_url: String,
    pub minimum_rebalance_lamports: u64,
    pub maximum_rebalance_lamports: u64,
    /// Minimum lamports to keep in the LST account for rent exempt
    pub minimum_lamports_reserves_lst_account: u64,
}

impl Default for MaxPoolOptions {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            minimum_rebalance_lamports: 1_000_000,
            maximum_rebalance_lamports: 1_000_000_000_000, // 1_000 SOL
            minimum_lamports_reserves_lst_account: 1_000_000,
        }
    }
}

impl MaxPoolOptions {
    pub fn deduct_reserves_lst_account_lamports(&self, current_lamports: u64) -> u64 {
        if current_lamports < self.minimum_lamports_reserves_lst_account {
            return 0;
        }
        current_lamports - self.minimum_lamports_reserves_lst_account
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_deduct_reserves_lst_account_lamports() {
        use super::MaxPoolOptions;

        let options = MaxPoolOptions {
            minimum_lamports_reserves_lst_account: 100,
            ..Default::default()
        };
        assert_eq!(options.deduct_reserves_lst_account_lamports(1000), 900);
        assert_eq!(options.deduct_reserves_lst_account_lamports(1), 0);
        assert_eq!(options.deduct_reserves_lst_account_lamports(0), 0);
    }
}

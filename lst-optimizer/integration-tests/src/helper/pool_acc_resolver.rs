#[async_trait::async_trait]
pub trait PoolAccountsResover {
    async fn fetch_spl_account_metas(&self, rpc: &RpcClient) -> Result<Vec<AccountMeta>>;
}

#[async_trait::async_trait]
impl PoolAccountsResover for Pubkey {
    async fn fetch_spl_account_metas(&self, rpc: &RpcClient) -> Result<Vec<AccountMeta>> {
        let pool_acc = rpc.get_account(&self).await?;
        let reso = SplLstSolCommonFreeArgsConst {
            spl_stake_pool: Keyed {
                account: pool_acc,
                pubkey: self.clone(),
            },
        };
        Ok(reso.resolve_spl_to_account_metas()?.to_vec())
    }
}

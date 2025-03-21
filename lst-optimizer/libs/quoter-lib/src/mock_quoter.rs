use crate::typedefs::QuoterClient;

pub struct MockQuoterClient {}

impl MockQuoterClient {
    pub fn new() -> Self {
        MockQuoterClient {}
    }
}

#[async_trait::async_trait]
impl QuoterClient for MockQuoterClient {
    fn from_parts(_rpc: solana_client::nonblocking::rpc_client::RpcClient) -> Self {
        MockQuoterClient {}
    }

    fn get_rpc_client(&self) -> &solana_client::nonblocking::rpc_client::RpcClient {
        unimplemented!()
    }

    async fn create_swap_instructions(
        &self,
        _swapper: &solana_sdk::pubkey::Pubkey,
        _src_mint: &solana_sdk::pubkey::Pubkey,
        _dst_mint: &solana_sdk::pubkey::Pubkey,
        _amount: u64,
        _min_amount_out: u64,
        _slippage_bps: Option<u16>,
    ) -> anyhow::Result<crate::typedefs::SwapInstructions> {
        unimplemented!()
    }
}

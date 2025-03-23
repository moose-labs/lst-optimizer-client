use anyhow::Result;
use quoter_lib::typedefs::{QuoterClient, SwapInstructions};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;

pub struct MockableQuoterClient {
    rpc: RpcClient,
    setup_instructions: Option<Vec<Instruction>>,
    swap_instructions: Option<Vec<Instruction>>,
}

impl MockableQuoterClient {
    pub fn with_setup_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.setup_instructions = Some(instructions);
        self
    }

    pub fn with_swap_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.swap_instructions = Some(instructions);
        self
    }
}

#[async_trait::async_trait]
impl QuoterClient for MockableQuoterClient {
    fn from_parts(rpc: solana_client::nonblocking::rpc_client::RpcClient) -> Self {
        MockableQuoterClient {
            rpc: rpc,
            setup_instructions: None,
            swap_instructions: None,
        }
    }

    fn get_rpc_client(&self) -> &solana_client::nonblocking::rpc_client::RpcClient {
        &self.rpc
    }

    async fn create_swap_instructions(
        &self,
        _swapper: &solana_sdk::pubkey::Pubkey,
        _src_mint: &solana_sdk::pubkey::Pubkey,
        _dst_mint: &solana_sdk::pubkey::Pubkey,
        _amount: u64,
        _min_amount_out: u64,
        _slippage_bps: Option<u16>,
    ) -> Result<SwapInstructions> {
        // This is a mock implementation, so we can just return a dummy transfer instruction
        Ok(SwapInstructions {
            setup_instructions: self.setup_instructions.clone().unwrap(),
            swap_instructions: self.swap_instructions.clone().unwrap(),
            cleanup_instructions: vec![],
            address_lookup_tables: vec![],
        })
    }
}

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{AddressLookupTableAccount, state::AddressLookupTable},
    instruction::Instruction,
    pubkey::Pubkey,
};

#[derive(Debug, Clone)]
pub enum SwapMode {
    ExactIn,
    ExactOut,
}

pub struct SwapInstructions {
    pub setup_instructions: Vec<Instruction>,
    pub swap_instructions: Vec<Instruction>,
    pub cleanup_instructions: Vec<Instruction>,
    pub address_lookup_tables: Vec<Pubkey>,
}

#[async_trait::async_trait]
pub trait QuoterClient: Sync + Send {
    fn from_parts(rpc: RpcClient) -> Self
    where
        Self: Sized;

    fn get_rpc_client(&self) -> &RpcClient;

    async fn create_swap_instructions(
        &self,
        swapper: &Pubkey,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        amount: u64,
        min_amount_out: u64,
        swap_mode: SwapMode,
        slippage_bps: Option<u16>,
    ) -> Result<SwapInstructions>;

    async fn resolve_address_lookup_table_accounts(
        &self,
        addresses: Vec<Pubkey>,
    ) -> Result<Vec<AddressLookupTableAccount>> {
        let rpc_client = self.get_rpc_client();
        let mut accounts = Vec::new();
        for key in addresses {
            if let Ok(account) = rpc_client.get_account(&key).await {
                if let Ok(address_lookup_table_account) =
                    AddressLookupTable::deserialize(&account.data)
                {
                    accounts.push(AddressLookupTableAccount {
                        key,
                        addresses: address_lookup_table_account.addresses.to_vec(),
                    });
                }
            }
        }
        Ok(accounts)
    }
}

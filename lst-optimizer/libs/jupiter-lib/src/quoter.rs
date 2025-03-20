use anyhow::Result;
pub use jupiter_swap_api_client::swap::SwapInstructionsResponse;
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount},
    pubkey::Pubkey,
};

const JUPITER_SWAP_API_URL: &str = "https://quote-api.jup.ag/v6";

pub type Instructions = SwapInstructionsResponse;

pub struct JupiterInstructionBuilder {
    client: JupiterSwapApiClient,
}

impl JupiterInstructionBuilder {
    pub fn new() -> Self {
        Self {
            client: JupiterSwapApiClient::new(JUPITER_SWAP_API_URL.to_string()),
        }
    }

    pub async fn create_jupiter_swap_instruction(
        &self,
        swapper: &Pubkey,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> Result<Instructions> {
        let jup_client = &self.client;
        let quote_request = QuoteRequest {
            input_mint: src_mint.clone(),
            output_mint: dst_mint.clone(),
            amount,
            slippage_bps: slippage_bps.unwrap_or(100),
            max_accounts: Some(16),
            only_direct_routes: Some(true),
            ..QuoteRequest::default()
        };

        let quote_res = jup_client.quote(&quote_request).await?;
        let swap_instructions = jup_client
            .swap_instructions(
                &(SwapRequest {
                    user_public_key: swapper.clone(),
                    quote_response: quote_res,
                    config: TransactionConfig::default(),
                }),
            )
            .await?;

        if swap_instructions.simulation_error.is_some() {
            return Err(anyhow::anyhow!(
                "error while create jupiter swap instructions: {:?}",
                swap_instructions.simulation_error
            ));
        }

        Ok(swap_instructions)
    }

    pub async fn get_address_lookup_table_accounts(
        &self,
        rpc_client: &RpcClient,
        addresses: Vec<Pubkey>,
    ) -> Result<Vec<AddressLookupTableAccount>> {
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

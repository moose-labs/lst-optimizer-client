use anyhow::Result;
pub use jupiter_swap_api_client::swap::SwapInstructionsResponse;
use jupiter_swap_api_client::{
    quote::{QuoteRequest, SwapMode as JupSwapMode},
    swap::SwapRequest,
    transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use quoter_lib::typedefs::{QuoterClient, SwapInstructions};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

const JUPITER_SWAP_API_URL: &str = "https://quote-api.jup.ag/v6";

pub struct JupiterQuoterClient {
    rpc: RpcClient,
    client: JupiterSwapApiClient,
}

impl JupiterQuoterClient {
    pub fn new(rpc_url: &str) -> Self {
        JupiterQuoterClient::from_parts(RpcClient::new(rpc_url.to_string()))
    }
}

#[async_trait::async_trait]
impl QuoterClient for JupiterQuoterClient {
    fn from_parts(rpc: RpcClient) -> Self {
        Self {
            rpc,
            client: JupiterSwapApiClient::new(JUPITER_SWAP_API_URL.to_string()),
        }
    }

    fn get_rpc_client(&self) -> &RpcClient {
        &self.rpc
    }

    async fn create_swap_instructions(
        &self,
        swapper: &Pubkey,
        receiver_token_account: &Pubkey,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        amount: u64,
        _min_amount_out: u64,
        slippage_bps: Option<u16>,
    ) -> Result<SwapInstructions> {
        let jup_client = &self.client;
        let quote_request = QuoteRequest {
            input_mint: src_mint.clone(),
            output_mint: dst_mint.clone(),
            amount,
            slippage_bps: slippage_bps.unwrap_or(3000),
            // only_direct_routes: Some(true),
            // max_accounts: Some(32),
            swap_mode: Some(JupSwapMode::ExactIn),
            ..QuoteRequest::default()
        };

        let quote_res = jup_client.quote(&quote_request).await?;
        let jup_instructions = jup_client
            .swap_instructions(
                &(SwapRequest {
                    user_public_key: swapper.clone(),
                    quote_response: quote_res,
                    config: TransactionConfig {
                        destination_token_account: Some(receiver_token_account.clone()),
                        ..TransactionConfig::default()
                    },
                }),
            )
            .await?;

        if jup_instructions.simulation_error.is_some() {
            return Err(anyhow::anyhow!(
                "error while create jupiter swap instructions: {:?}",
                jup_instructions.simulation_error
            ));
        }

        let mut ret = SwapInstructions {
            setup_instructions: jup_instructions.setup_instructions,
            swap_instructions: vec![jup_instructions.swap_instruction],
            cleanup_instructions: vec![],
            address_lookup_tables: jup_instructions.address_lookup_table_addresses,
        };

        if jup_instructions.cleanup_instruction.is_some() {
            ret.cleanup_instructions
                .push(jup_instructions.cleanup_instruction.unwrap());
        }

        Ok(ret)
    }
}

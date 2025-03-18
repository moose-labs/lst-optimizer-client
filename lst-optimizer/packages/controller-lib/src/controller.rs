use anyhow::Result;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig,
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{
    instruction::Instruction,
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status::UiTransactionReturnData;

solana_program::declare_id!("43vcPfe8ThRLwfJqhXoM2KwqmpqQK1wCrfvZsxrULsbQ");

pub struct ControllerClient {
    rpc_client: RpcClient,
}

impl ControllerClient {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self { rpc_client }
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    // Invoke the instructions simulation on the RPC client and return the return data

    pub async fn simulate_instructions(
        &self,
        payer: &Pubkey,
        instructions: &[Instruction],
    ) -> Result<UiTransactionReturnData> {
        let rpc = self.rpc_client();
        let blockhash: solana_sdk::hash::Hash = rpc.get_latest_blockhash().await?;
        let message =
            VersionedMessage::V0(Message::try_compile(&payer, instructions, &[], blockhash)?);
        let tx: VersionedTransaction = VersionedTransaction {
            signatures: vec![Signature::default(); message.header().num_required_signatures.into()],
            message,
        };

        let config = RpcSimulateTransactionConfig {
            sig_verify: false,
            ..Default::default()
        };

        let RpcSimulateTransactionResult {
            return_data, err, ..
        } = rpc
            .simulate_transaction_with_config(&tx, config)
            .await?
            .value;
        if let Some(err) = err {
            return Err(anyhow::anyhow!("error in simulation: {:?}", err));
        }
        match return_data {
            Some(data) => Ok(data),
            None => Err(anyhow::anyhow!("invalid return data")),
        }
    }
}

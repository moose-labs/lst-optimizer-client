use anyhow::Result;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig},
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    commitment_config::CommitmentLevel,
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

    async fn build_transaction(
        &self,
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let recent_blockhash: solana_sdk::hash::Hash =
            self.rpc_client.get_latest_blockhash().await?;
        let message = VersionedMessage::V0(Message::try_compile(
            &payer,
            instructions,
            address_lookup_table_accounts,
            recent_blockhash,
        )?);
        let tx: VersionedTransaction = VersionedTransaction {
            signatures: vec![Signature::default(); message.header().num_required_signatures.into()],
            message: message.clone(),
        };
        Ok(tx)
    }

    pub async fn invoke_instructions(
        &self,
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<Signature> {
        let rpc = self.rpc_client();
        let tx = self
            .build_transaction(payer, instructions, address_lookup_table_accounts)
            .await?;
        let ret = rpc
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    preflight_commitment: Some(CommitmentLevel::Finalized),
                    ..Default::default()
                },
            )
            .await?;
        Ok(ret)
    }

    pub async fn simulate_instructions(
        &self,
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<RpcSimulateTransactionResult> {
        let rpc = self.rpc_client();
        let tx = self
            .build_transaction(payer, instructions, address_lookup_table_accounts)
            .await?;
        let ret = rpc
            .simulate_transaction_with_config(
                &tx,
                RpcSimulateTransactionConfig {
                    sig_verify: false,
                    ..Default::default()
                },
            )
            .await?;
        Ok(ret.value)
    }

    pub async fn simulate_returned_from_instructions(
        &self,
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<UiTransactionReturnData> {
        let RpcSimulateTransactionResult {
            return_data, err, ..
        } = self
            .simulate_instructions(payer, instructions, address_lookup_table_accounts)
            .await?;
        if let Some(err) = err {
            return Err(anyhow::anyhow!("error in simulation: {:?}", err));
        }
        match return_data {
            Some(data) => Ok(data),
            None => Err(anyhow::anyhow!("invalid return data")),
        }
    }
}

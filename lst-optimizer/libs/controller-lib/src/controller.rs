use anyhow::Result;
use base64::Engine;
use lst_optimizer_utils::logger::info;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig,
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    instruction::Instruction,
    message::{v0::Message, VersionedMessage},
    signature::{Keypair, Signature},
    transaction::VersionedTransaction,
};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signer};
use solana_transaction_status::UiTransactionReturnData;

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
        payer: &Keypair,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let rpc = self.rpc_client();
        let recent_blockhash = rpc.get_latest_blockhash().await?;
        let compiled_message = Message::try_compile(
            &payer.pubkey(),
            &instructions,
            address_lookup_table_accounts,
            recent_blockhash,
        )?;

        // Create base64 encoded transaction for debugging
        let tx_base64 =
            base64::prelude::BASE64_STANDARD.encode(bincode::serialize(&compiled_message)?);
        info!("{}", tx_base64);

        let tx = VersionedTransaction::try_new(VersionedMessage::V0(compiled_message), &[payer])?;
        Ok(tx)
    }

    pub async fn invoke_instructions(
        &self,
        payer: &Keypair,
        instructions: &[Instruction],
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> Result<Signature> {
        let rpc = self.rpc_client();
        let tx = self
            .build_transaction(payer, instructions, address_lookup_table_accounts)
            .await?;
        let ret = rpc.send_transaction(&tx).await?;
        Ok(ret)
    }

    pub async fn simulate_instructions(
        &self,
        payer: &Keypair,
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
                    commitment: Some(CommitmentConfig::processed()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(ret.value)
    }

    pub async fn simulate_returned_from_instructions(
        &self,
        payer: &Keypair,
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

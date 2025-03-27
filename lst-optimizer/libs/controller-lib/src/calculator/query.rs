use sanctum_token_ratio::U64ValueRange;

use anyhow::Result;
use solana_sdk::signature::Keypair;

use crate::controller::ControllerClient;

use super::{
    helper::parse_u64_value_range_return_data, instructions::CalculatorInstructions,
    typedefs::CalculatorType,
};

// Convert between LST and SOL using the calculator program

#[async_trait::async_trait]
pub trait CalculatorQuery {
    async fn convert_sol_to_lst(
        &self,
        payer: &Keypair,
        calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<U64ValueRange>;

    async fn convert_lst_to_sol(
        &self,
        payer: &Keypair,
        calculator_type: CalculatorType,
        amount: u64,
    ) -> Result<U64ValueRange>;
}

#[async_trait::async_trait]
impl CalculatorQuery for ControllerClient {
    async fn convert_sol_to_lst(
        &self,
        payer: &Keypair,
        calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<U64ValueRange> {
        let accounts = calculator_type
            .fetch_account_metas(self.rpc_client())
            .await?;
        let instruction =
            self.create_sol_to_lst_instruction(calculator_type, lamports, accounts)?;
        let return_data = self
            .simulate_returned_from_instructions(payer, &[instruction], &[])
            .await?;
        let val = parse_u64_value_range_return_data(&return_data)?;
        Ok(val)
    }

    async fn convert_lst_to_sol(
        &self,
        payer: &Keypair,
        calculator_type: CalculatorType,
        amount: u64,
    ) -> Result<U64ValueRange> {
        let accounts = calculator_type
            .fetch_account_metas(self.rpc_client())
            .await?;
        let instruction = self.create_lst_to_sol_instruction(calculator_type, amount, accounts)?;
        let return_data = self
            .simulate_returned_from_instructions(payer, &[instruction], &[])
            .await?;
        let val = parse_u64_value_range_return_data(&return_data)?;
        Ok(val)
    }
}

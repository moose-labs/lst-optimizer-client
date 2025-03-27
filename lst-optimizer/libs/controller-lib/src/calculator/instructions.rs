use anyhow::Result;
use generic_pool_calculator_interface::{
    lst_to_sol_ix_with_program_id, sol_to_lst_ix_with_program_id, LstToSolIxArgs, LstToSolKeys,
    SolToLstIxArgs, SolToLstKeys,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::controller::ControllerClient;

use super::typedefs::{calculator_program_id, CalculatorType};

pub trait CalculatorInstructions {
    fn create_lst_to_sol_instruction(
        &self,
        calculator_type: CalculatorType,
        amount: u64,
        accounts: Vec<AccountMeta>,
    ) -> Result<Instruction>;

    fn create_sol_to_lst_instruction(
        &self,
        calculator_type: CalculatorType,
        lamports: u64,
        accounts: Vec<AccountMeta>,
    ) -> Result<Instruction>;
}

impl CalculatorInstructions for ControllerClient {
    fn create_sol_to_lst_instruction(
        &self,
        calculator_type: CalculatorType,
        lamports: u64,
        accounts: Vec<AccountMeta>,
    ) -> Result<Instruction> {
        let program_id = calculator_program_id(&calculator_type);
        let mut instruction = sol_to_lst_ix_with_program_id(
            program_id,
            SolToLstKeys {
                // keys will all be replaced by lst_sol_common_account_metas
                lst_mint: Pubkey::default(),
                state: Pubkey::default(),
                pool_state: Pubkey::default(),
                pool_program: Pubkey::default(),
                pool_program_data: Pubkey::default(),
            },
            SolToLstIxArgs { amount: lamports },
        )?;
        instruction.accounts = accounts;
        Ok(instruction)
    }

    fn create_lst_to_sol_instruction(
        &self,
        calculator_type: CalculatorType,
        amount: u64,
        accounts: Vec<AccountMeta>,
    ) -> Result<Instruction> {
        let program_id = calculator_program_id(&calculator_type);
        let mut instruction = lst_to_sol_ix_with_program_id(
            program_id,
            LstToSolKeys {
                // keys will all be replaced by lst_sol_common_account_metas
                lst_mint: Pubkey::default(),
                state: Pubkey::default(),
                pool_state: Pubkey::default(),
                pool_program: Pubkey::default(),
                pool_program_data: Pubkey::default(),
            },
            LstToSolIxArgs { amount },
        )?;
        instruction.accounts = accounts;
        Ok(instruction)
    }
}

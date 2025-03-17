use anyhow::Result;
use flat_fee_lib::account_resolvers::PriceExactInFreeArgs;
use s_controller_lib::{
    end_rebalance_ix_from_start_rebalance_ix,
    start_rebalance_ix_by_mints_full_for_prog,
    swap_exact_in_ix_by_mint_full,
    SrcDstLstSolValueCalcAccountSuffixes,
    StartRebalanceByMintsFreeArgs,
    StartRebalanceIxLstAmts,
    SwapByMintsFreeArgs,
    SwapExactInAmounts,
};
use solana_readonly_account::keyed::Keyed;
use solana_sdk::{ account::Account, instruction::{ AccountMeta, Instruction }, pubkey::Pubkey };
use stakedex_interface::{
    StakeWrappedSolIxArgs,
    StakeWrappedSolIxData,
    StakeWrappedSolKeys,
    STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN,
};

use crate::{
    controller_instructions::ControllerInstructionBuilder,
    mint::typedefs::MintWithTokenProgram,
};

pub trait RebalancingInstructions {
    fn create_start_rebalance_instruction(
        &self,
        program_id: &Pubkey,
        withdraw_to: &Pubkey, // the account who will receive the withdrawn funds
        pool_state_addr: &Pubkey,
        pool_state: &Account,
        lst_state_addr: &Pubkey,
        lst_state: &Account,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_mint_program: &Pubkey,
        dst_mint_program: &Pubkey,
        src_accounts: Vec<AccountMeta>,
        dst_accounts: Vec<AccountMeta>,
        lamports: u64
    ) -> Result<Instruction>;
    fn create_end_rebalance_instruction_from_start(
        &self,
        start_rebalance_ix: &Instruction
    ) -> Result<Instruction>;
    fn create_stake_wrapped_sol_instruction(
        &self,
        program_id: &Pubkey,
        accounts: StakeWrappedSolKeys,
        args: StakeWrappedSolIxArgs
    ) -> Result<Instruction>;
    fn create_sanctum_swap_exact_in_instruction(
        &self,
        swapper: &Pubkey,
        swapper_src_acc: &Pubkey,
        swapper_dst_acc: &Pubkey,
        lst_state: &Account,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_mint_program: &Pubkey,
        dst_mint_program: &Pubkey,
        src_accounts: Vec<AccountMeta>,
        dst_accounts: Vec<AccountMeta>,
        amount: u64,
        min_amount_out: u64
    ) -> Result<Instruction>;
}

impl RebalancingInstructions for ControllerInstructionBuilder {
    fn create_start_rebalance_instruction(
        &self,
        program_id: &Pubkey,
        withdraw_to: &Pubkey, // the account who will receive the withdrawn funds
        pool_state_addr: &Pubkey,
        pool_state: &Account,
        lst_state_addr: &Pubkey,
        lst_state: &Account,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_mint_program: &Pubkey,
        dst_mint_program: &Pubkey,
        src_accounts: Vec<AccountMeta>,
        dst_accounts: Vec<AccountMeta>,
        lamports: u64
    ) -> Result<Instruction> {
        let instruction = start_rebalance_ix_by_mints_full_for_prog(
            program_id.clone(),
            StartRebalanceByMintsFreeArgs {
                withdraw_to: withdraw_to.clone(),
                pool_state: Keyed {
                    pubkey: pool_state_addr.clone(),
                    account: pool_state,
                },
                lst_state_list: Keyed {
                    pubkey: lst_state_addr.clone(),
                    account: lst_state,
                },
                src_lst_mint: MintWithTokenProgram {
                    pubkey: src_mint.clone(),
                    token_program: src_mint_program.clone(),
                },
                dst_lst_mint: MintWithTokenProgram {
                    pubkey: dst_mint.clone(),
                    token_program: dst_mint_program.clone(),
                },
            },
            StartRebalanceIxLstAmts {
                amount: lamports,
                min_starting_src_lst: 0,
                max_starting_dst_lst: u64::MAX,
            },
            SrcDstLstSolValueCalcAccountSuffixes {
                src_lst_calculator_accounts: &src_accounts,
                dst_lst_calculator_accounts: &dst_accounts,
            }
        )?;
        Ok(instruction)
    }

    fn create_end_rebalance_instruction_from_start(
        &self,
        start_rebalance_ix: &Instruction
    ) -> Result<Instruction> {
        Ok(end_rebalance_ix_from_start_rebalance_ix(start_rebalance_ix)?)
    }

    // This function is used to create a stake wrapped sol instruction for the sanctum program

    fn create_stake_wrapped_sol_instruction(
        &self,
        program_id: &Pubkey,
        accounts: StakeWrappedSolKeys,
        args: StakeWrappedSolIxArgs
    ) -> Result<Instruction> {
        let metas: [AccountMeta; STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN] = accounts.into();
        let data: StakeWrappedSolIxData = args.into();
        Ok(Instruction {
            program_id: program_id.clone(),
            accounts: Vec::from(metas),
            data: data.try_to_vec()?,
        })
    }

    // This function is used to create a swap instruction for the sanctum program

    fn create_sanctum_swap_exact_in_instruction(
        &self,
        swapper: &Pubkey,
        swapper_src_acc: &Pubkey,
        swapper_dst_acc: &Pubkey,
        lst_state: &Account,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_mint_program: &Pubkey,
        dst_mint_program: &Pubkey,
        src_accounts: Vec<AccountMeta>,
        dst_accounts: Vec<AccountMeta>,
        amount: u64,
        min_amount_out: u64
    ) -> Result<Instruction> {
        let instruction = swap_exact_in_ix_by_mint_full(
            SwapByMintsFreeArgs {
                signer: swapper.clone(),
                src_lst_acc: swapper_src_acc.clone(),
                dst_lst_acc: swapper_dst_acc.clone(),
                src_lst_mint: MintWithTokenProgram {
                    pubkey: src_mint.clone(),
                    token_program: src_mint_program.clone(),
                },
                dst_lst_mint: MintWithTokenProgram {
                    pubkey: dst_mint.clone(),
                    token_program: dst_mint_program.clone(),
                },
                lst_state_list: lst_state,
            },
            SwapExactInAmounts {
                amount,
                min_amount_out,
            },
            SrcDstLstSolValueCalcAccountSuffixes {
                src_lst_calculator_accounts: &src_accounts,
                dst_lst_calculator_accounts: &dst_accounts,
            },
            &(PriceExactInFreeArgs {
                input_lst_mint: src_mint.clone(),
                output_lst_mint: dst_mint.clone(),
            }).resolve_to_account_metas(),
            flat_fee_lib::program::ID
        )?;

        Ok(instruction)
    }
}

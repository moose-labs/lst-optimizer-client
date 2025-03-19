use anyhow::Result;
use flat_fee_lib::account_resolvers::PriceExactInFreeArgs;
use s_controller_lib::{
    end_rebalance_ix_from_start_rebalance_ix, start_rebalance_ix_by_mints_full_for_prog,
    swap_exact_in_ix_by_mint_full, SrcDstLstSolValueCalcAccountSuffixes,
    StartRebalanceByMintsFreeArgs, StartRebalanceIxLstAmts, SwapByMintsFreeArgs,
    SwapExactInAmounts,
};
use solana_readonly_account::keyed::Keyed;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_helper::mint::MintAccountQuery;
use stakedex_interface::{
    StakeWrappedSolIxArgs, StakeWrappedSolIxData, StakeWrappedSolKeys,
    STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN,
};

use crate::{
    calculator::{query::CalculatorQuery, typedefs::CalculatorType},
    controller::ControllerClient,
    mint::typedefs::MintWithTokenProgram,
    state::PoolQuery,
};

#[async_trait::async_trait]
pub trait RebalancingInstructions {
    async fn create_start_rebalance_instruction(
        &self,
        program_id: &Pubkey,
        withdraw_to: &Pubkey, // the account who will receive the withdrawn funds
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_calculator_type: CalculatorType,
        dst_calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<Instruction>;
    async fn create_end_rebalance_instruction_from_start(
        &self,
        start_rebalance_ix: &Instruction,
    ) -> Result<Instruction>;
    async fn create_stake_wrapped_sol_instruction(
        &self,
        program_id: &Pubkey,
        accounts: StakeWrappedSolKeys,
        args: StakeWrappedSolIxArgs,
    ) -> Result<Instruction>;
    async fn create_sanctum_swap_exact_in_instruction(
        &self,
        program_id: &Pubkey,
        swapper: &Pubkey,
        swapper_src_acc: &Pubkey,
        swapper_dst_acc: &Pubkey,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_calculator_type: CalculatorType,
        dst_calculator_type: CalculatorType,
        amount: u64,
        min_amount_out: u64,
    ) -> Result<Instruction>;
}

#[async_trait::async_trait]
impl RebalancingInstructions for ControllerClient {
    async fn create_start_rebalance_instruction(
        &self,
        program_id: &Pubkey,
        withdraw_to: &Pubkey, // the account who will receive the withdrawn funds
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_calculator_type: CalculatorType,
        dst_calculator_type: CalculatorType,
        lamports: u64,
    ) -> Result<Instruction> {
        let rpc = self.rpc_client();
        let pool_state_addr = self.get_pool_state_address(program_id).await;
        let lst_state_addr = self.get_lst_state_list_address(program_id).await;

        let src_token_program = src_mint.get_mint_owner(rpc).await?;
        let dst_token_program = dst_mint.get_mint_owner(rpc).await?;

        let src_accs = self
            .fetch_calculator_account_metas(&src_calculator_type)
            .await?;
        let dst_accs = self
            .fetch_calculator_account_metas(&dst_calculator_type)
            .await?;

        let instruction = start_rebalance_ix_by_mints_full_for_prog(
            program_id.clone(),
            StartRebalanceByMintsFreeArgs {
                withdraw_to: withdraw_to.clone(),
                pool_state: Keyed {
                    pubkey: pool_state_addr,
                    account: self.get_pool_state_account(&pool_state_addr).await?,
                },
                lst_state_list: Keyed {
                    pubkey: lst_state_addr,
                    account: self.get_lst_state_list_account(&lst_state_addr).await?,
                },
                src_lst_mint: MintWithTokenProgram {
                    pubkey: src_mint.clone(),
                    token_program: src_token_program,
                },
                dst_lst_mint: MintWithTokenProgram {
                    pubkey: dst_mint.clone(),
                    token_program: dst_token_program,
                },
            },
            StartRebalanceIxLstAmts {
                amount: lamports,
                min_starting_src_lst: 0,
                max_starting_dst_lst: u64::MAX,
            },
            SrcDstLstSolValueCalcAccountSuffixes {
                src_lst_calculator_accounts: &src_accs,
                dst_lst_calculator_accounts: &dst_accs,
            },
        )?;

        Ok(instruction)
    }

    async fn create_end_rebalance_instruction_from_start(
        &self,
        start_rebalance_ix: &Instruction,
    ) -> Result<Instruction> {
        Ok(end_rebalance_ix_from_start_rebalance_ix(
            start_rebalance_ix,
        )?)
    }

    // This function is used to create a stake wrapped sol instruction for the sanctum program

    async fn create_stake_wrapped_sol_instruction(
        &self,
        program_id: &Pubkey,
        accounts: StakeWrappedSolKeys,
        args: StakeWrappedSolIxArgs,
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

    async fn create_sanctum_swap_exact_in_instruction(
        &self,
        program_id: &Pubkey,
        swapper: &Pubkey,
        swapper_src_acc: &Pubkey,
        swapper_dst_acc: &Pubkey,
        src_mint: &Pubkey,
        dst_mint: &Pubkey,
        src_calculator_type: CalculatorType,
        dst_calculator_type: CalculatorType,
        amount: u64,
        min_amount_out: u64,
    ) -> Result<Instruction> {
        let rpc = self.rpc_client();
        let lst_state_addr = self.get_lst_state_list_address(program_id).await;
        let lst_state = self.get_lst_state_list_account(&lst_state_addr).await?;
        let instruction = swap_exact_in_ix_by_mint_full(
            SwapByMintsFreeArgs {
                signer: swapper.clone(),
                src_lst_acc: swapper_src_acc.clone(),
                dst_lst_acc: swapper_dst_acc.clone(),
                src_lst_mint: MintWithTokenProgram {
                    pubkey: src_mint.clone(),
                    token_program: src_mint.get_mint_owner(rpc).await?,
                },
                dst_lst_mint: MintWithTokenProgram {
                    pubkey: dst_mint.clone(),
                    token_program: dst_mint.get_mint_owner(rpc).await?,
                },
                lst_state_list: lst_state,
            },
            SwapExactInAmounts {
                amount,
                min_amount_out,
            },
            SrcDstLstSolValueCalcAccountSuffixes {
                src_lst_calculator_accounts: &self
                    .fetch_calculator_account_metas(&src_calculator_type)
                    .await?,
                dst_lst_calculator_accounts: &self
                    .fetch_calculator_account_metas(&dst_calculator_type)
                    .await?,
            },
            &(PriceExactInFreeArgs {
                input_lst_mint: src_mint.clone(),
                output_lst_mint: dst_mint.clone(),
            })
            .resolve_to_account_metas(),
            flat_fee_lib::program::ID, // TODO: use Moose's forked program ID
        )?;

        Ok(instruction)
    }
}

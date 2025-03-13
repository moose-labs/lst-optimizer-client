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
use solana_client::rpc_client::RpcClient;
use solana_readonly_account::keyed::Keyed;
use solana_sdk::{ instruction::{ AccountMeta, Instruction }, pubkey::Pubkey };
use stakedex_interface::{
    StakeWrappedSolIxArgs,
    StakeWrappedSolIxData,
    StakeWrappedSolKeys,
    STAKE_WRAPPED_SOL_IX_ACCOUNTS_LEN,
};

use crate::{
    calculator::CalculatorType,
    mint::{ MintAccountResolver, MintWithTokenProgram },
    state::{ find_lst_state_list_address, find_pool_state_address },
};

pub fn create_start_rebalance_instruction(
    rpc: &RpcClient,
    program_id: &Pubkey,
    withdraw_to: &Pubkey, // the account who will receive the withdrawn funds
    src_mint: &Pubkey,
    dst_mint: &Pubkey,
    src_cal_type: &CalculatorType,
    dst_cal_type: &CalculatorType,
    lamports: u64
) -> Result<Instruction> {
    let pool_state_addr = find_pool_state_address(program_id);
    let pool_state = rpc.get_account(&pool_state_addr)?;

    let lst_state_list_addr = find_lst_state_list_address(program_id);
    let lst_state_list_acc = rpc.get_account(&lst_state_list_addr)?;

    let instruction = start_rebalance_ix_by_mints_full_for_prog(
        program_id.clone(),
        StartRebalanceByMintsFreeArgs {
            withdraw_to: withdraw_to.clone(),
            pool_state: Keyed {
                pubkey: pool_state_addr,
                account: &pool_state,
            },
            lst_state_list: Keyed {
                pubkey: lst_state_list_addr,
                account: &lst_state_list_acc,
            },
            src_lst_mint: MintWithTokenProgram {
                pubkey: src_mint.clone(),
                token_program: src_mint.resolve_owner(rpc)?,
            },
            dst_lst_mint: MintWithTokenProgram {
                pubkey: dst_mint.clone(),
                token_program: dst_mint.resolve_owner(rpc)?,
            },
        },
        StartRebalanceIxLstAmts {
            amount: lamports,
            min_starting_src_lst: 0,
            max_starting_dst_lst: u64::MAX,
        },
        SrcDstLstSolValueCalcAccountSuffixes {
            src_lst_calculator_accounts: &src_cal_type.resolve_account_metas(rpc)?,
            dst_lst_calculator_accounts: &dst_cal_type.resolve_account_metas(rpc)?,
        }
    )?;
    Ok(instruction)
}

pub fn create_end_rebalance_instruction_from_start(
    start_rebalance_ix: &Instruction
) -> Result<Instruction> {
    Ok(end_rebalance_ix_from_start_rebalance_ix(start_rebalance_ix)?)
}

// deposit, swap instructions

pub fn create_stake_wrapped_sol_instruction(
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

pub fn create_sanctum_swap_exact_in_instruction(
    rpc: &RpcClient,
    program_id: &Pubkey, // controller
    // the swapper infos
    swapper: &Pubkey,
    swapper_src_acc: &Pubkey,
    swapper_dst_acc: &Pubkey,
    // mint infos
    src_mint: &Pubkey,
    dst_mint: &Pubkey,
    // src & dst types
    src_cal_type: &CalculatorType,
    dst_cal_type: &CalculatorType,
    // swap parameters
    amount: u64,
    min_amount_out: u64
) -> Result<Instruction> {
    let lst_state_list_addr = find_lst_state_list_address(program_id);
    let lst_state_list_acc = rpc.get_account(&lst_state_list_addr)?;

    let instruction = swap_exact_in_ix_by_mint_full(
        SwapByMintsFreeArgs {
            signer: swapper.clone(),
            src_lst_acc: swapper_src_acc.clone(),
            dst_lst_acc: swapper_dst_acc.clone(),
            src_lst_mint: MintWithTokenProgram {
                pubkey: src_mint.clone(),
                token_program: src_mint.resolve_owner(rpc)?,
            },
            dst_lst_mint: MintWithTokenProgram {
                pubkey: dst_mint.clone(),
                token_program: dst_mint.resolve_owner(rpc)?,
            },
            lst_state_list: lst_state_list_acc,
        },
        SwapExactInAmounts {
            amount,
            min_amount_out,
        },
        SrcDstLstSolValueCalcAccountSuffixes {
            src_lst_calculator_accounts: &src_cal_type.resolve_account_metas(rpc)?,
            dst_lst_calculator_accounts: &dst_cal_type.resolve_account_metas(rpc)?,
        },
        &(PriceExactInFreeArgs {
            input_lst_mint: src_mint.clone(),
            output_lst_mint: dst_mint.clone(),
        }).resolve_to_account_metas(),
        flat_fee_lib::program::ID
    )?;

    Ok(instruction)
}

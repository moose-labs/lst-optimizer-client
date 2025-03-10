use borsh::BorshDeserialize;
use data_encoding::BASE64;
use generic_pool_calculator_interface::{
    lst_to_sol_ix_with_program_id,
    sol_to_lst_ix_with_program_id,
    LstToSolIxArgs,
    LstToSolKeys,
    SolToLstIxArgs,
    SolToLstKeys,
};
use lido_calculator_lib::lido_sol_val_calc_account_metas;
use marinade_calculator_lib::marinade_sol_val_calc_account_metas;
use sanctum_token_ratio::U64ValueRange;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSimulateTransactionConfig,
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{
    instruction::{ AccountMeta, Instruction },
    message::{ v0::Message, VersionedMessage },
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};
use anyhow::Result;
use solana_readonly_account::keyed::Keyed;
use solana_transaction_status::UiTransactionReturnData;
use spl_calculator_lib::SplLstSolCommonFreeArgsConst;
use wsol_calculator_lib::WSOL_LST_SOL_COMMON_METAS;

#[derive(Clone, Debug)]
pub enum CalculatorType {
    Lido,
    Marinade,
    Wsol,
    Spl(String),
    SanctumSpl(String),
    SanctumSplMulti(String),
}

// Convert between LST and SOL using the calculator program

pub fn convert_sol_to_lst(
    rpc: &RpcClient,
    payer: &Pubkey,
    calculator_type: CalculatorType,
    lamports: u64
) -> Result<U64ValueRange> {
    let instruction = create_sol_to_lst_instruction(rpc, calculator_type, lamports)?;
    let return_data = invoke_instructions(rpc, payer, &[instruction])?;
    let val = parse_u64_value_range_return_data(&return_data)?;
    Ok(val)
}

pub fn convert_lst_to_sol(
    rpc: &RpcClient,
    payer: &Pubkey,
    calculator_type: CalculatorType,
    amount: u64
) -> Result<U64ValueRange> {
    let instruction = create_lst_to_sol_instruction(rpc, calculator_type, amount)?;
    let return_data = invoke_instructions(rpc, payer, &[instruction])?;
    let val = parse_u64_value_range_return_data(&return_data)?;
    Ok(val)
}

// Invoke the instructions on the RPC client and return the return data

fn invoke_instructions(
    rpc: &RpcClient,
    payer: &Pubkey,
    instructions: &[Instruction]
) -> Result<UiTransactionReturnData> {
    let blockhash = rpc.get_latest_blockhash()?;
    let message = VersionedMessage::V0(Message::try_compile(payer, instructions, &[], blockhash)?);
    let tx: VersionedTransaction = VersionedTransaction {
        signatures: vec![Signature::default(); message.header().num_required_signatures.into()],
        message,
    };
    let config = RpcSimulateTransactionConfig {
        sig_verify: false,
        ..Default::default()
    };
    let RpcSimulateTransactionResult { return_data, err, .. } =
        rpc.simulate_transaction_with_config(&tx, config)?.value;
    if let Some(err) = err {
        return Err(anyhow::anyhow!("error in simulation: {:?}", err));
    }
    match return_data {
        Some(data) => Ok(data),
        None => Err(anyhow::anyhow!("invalid return data")),
    }
}

// Helper function to parse the return data

fn parse_u64_value_range_return_data(
    return_data: &UiTransactionReturnData
) -> Result<U64ValueRange> {
    let UiTransactionReturnData { data: (data_str, _), .. } = return_data;
    let data = BASE64.decode(data_str.as_bytes())?;
    let range = U64ValueRange::deserialize(&mut data.as_ref())?;
    Ok(range)
}

// Helper functions to create the instructions

fn create_sol_to_lst_instruction(
    rpc: &RpcClient,
    calculator_type: CalculatorType,
    lamports: u64
) -> Result<Instruction> {
    let program_id = calculator_program_id(&calculator_type);
    let mut instruction = sol_to_lst_ix_with_program_id(
        program_id,
        SolToLstKeys { // keys will all be replaced by lst_sol_common_account_metas
            lst_mint: Pubkey::default(),
            state: Pubkey::default(),
            pool_state: Pubkey::default(),
            pool_program: Pubkey::default(),
            pool_program_data: Pubkey::default(),
        },
        SolToLstIxArgs {
            amount: lamports,
        }
    )?;
    instruction.accounts = fetch_calculator_account_metas(rpc, &calculator_type)?;
    Ok(instruction)
}

fn create_lst_to_sol_instruction(
    rpc: &RpcClient,
    calculator_type: CalculatorType,
    amount: u64
) -> Result<Instruction> {
    let program_id = calculator_program_id(&calculator_type);
    let mut instruction = lst_to_sol_ix_with_program_id(
        program_id,
        LstToSolKeys { // keys will all be replaced by lst_sol_common_account_metas
            lst_mint: Pubkey::default(),
            state: Pubkey::default(),
            pool_state: Pubkey::default(),
            pool_program: Pubkey::default(),
            pool_program_data: Pubkey::default(),
        },
        LstToSolIxArgs {
            amount,
        }
    )?;
    instruction.accounts = fetch_calculator_account_metas(rpc, &calculator_type)?;
    Ok(instruction)
}

// Metadata for the common accounts used by all calculators

pub fn calculator_program_id(calculator_type: &CalculatorType) -> Pubkey {
    match calculator_type {
        CalculatorType::Lido => lido_calculator_lib::program::ID,
        CalculatorType::Marinade => marinade_calculator_lib::program::ID,
        CalculatorType::Wsol => wsol_calculator_lib::program::ID,
        CalculatorType::Spl(_) => spl_calculator_lib::program::ID,
        CalculatorType::SanctumSpl(_) => spl_calculator_lib::sanctum_spl_sol_val_calc_program::ID,
        CalculatorType::SanctumSplMulti(_) => {
            spl_calculator_lib::sanctum_spl_multi_sol_val_calc_program::ID
        }
    }
}

fn fetch_calculator_account_metas(
    rpc: &RpcClient,
    calculator_type: &CalculatorType
) -> Result<Vec<AccountMeta>> {
    let accs = match calculator_type {
        CalculatorType::Lido => lido_sol_val_calc_account_metas().to_vec(),
        CalculatorType::Marinade => marinade_sol_val_calc_account_metas().to_vec(),
        CalculatorType::Wsol => WSOL_LST_SOL_COMMON_METAS.to_vec(),
        | CalculatorType::Spl(pool)
        | CalculatorType::SanctumSpl(pool)
        | CalculatorType::SanctumSplMulti(pool) => {
            let pool: Pubkey = pool.parse()?;
            let pool_acc = rpc.get_account(&pool)?;
            let reso = SplLstSolCommonFreeArgsConst {
                spl_stake_pool: Keyed {
                    account: pool_acc,
                    pubkey: pool,
                },
            };
            (match calculator_type {
                CalculatorType::Spl(_) => reso.resolve_spl_to_account_metas(),
                CalculatorType::SanctumSpl(_) => reso.resolve_sanctum_spl_to_account_metas(),
                CalculatorType::SanctumSplMulti(_) =>
                    reso.resolve_sanctum_spl_multi_to_account_metas(),
                _ => unreachable!(),
            })?.to_vec()
        }
    };

    Ok(accs)
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey;

    use super::*;

    #[test]
    fn test_calculators() {
        let url =
            "https://mainnet.helius-rpc.com/?api-key=f48894c7-d3cd-406a-8bec-bc29c3f9052e".to_string();
        let rpc = solana_client::rpc_client::RpcClient::new(url);

        let amount_sol = convert_lst_to_sol(
            &rpc,
            &pubkey!("86naSVEnAUH1C9b4WktPqohydNhW5c1Tnt2foQqnZKb1"),
            CalculatorType::Marinade,
            1_000_000_000
        );
        println!("1 mSOL = {:?} SOL", amount_sol.unwrap().get_min());

        let amount_lst = convert_sol_to_lst(
            &rpc,
            &pubkey!("86naSVEnAUH1C9b4WktPqohydNhW5c1Tnt2foQqnZKb1"),
            CalculatorType::Marinade,
            1_000_000_000
        );
        println!("1 SOL = {:?} mSOL", amount_lst.unwrap().get_min());
    }
}

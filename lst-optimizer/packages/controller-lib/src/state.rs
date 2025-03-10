use anyhow::Result;
pub use s_controller_interface::{ LstState, PoolState };
use s_controller_lib::{
    create_pool_reserves_address_with_pool_state_id,
    create_protocol_fee_accumulator_address_with_protocol_fee_id,
    find_lst_state_list_address as _find_lst_state_list_address,
    find_pool_state_address as _find_pool_state_address,
    try_lst_state_list,
    try_pool_state,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub fn find_pool_state_address(program_id: &Pubkey) -> Pubkey {
    _find_pool_state_address(program_id.clone()).0
}

pub fn get_pool_state(rpc: &RpcClient, pool_state_addr: &Pubkey) -> Result<PoolState> {
    let pool_state_acc = rpc.get_account(&pool_state_addr)?;
    let pool_state = try_pool_state(&pool_state_acc.data)?;
    Ok(*pool_state)
}

pub fn find_and_get_pool_state(rpc: &RpcClient, program_id: &Pubkey) -> Result<PoolState> {
    let pool_state_addr = find_pool_state_address(program_id);
    get_pool_state(rpc, &pool_state_addr)
}

pub fn find_lst_state_list_address(program_id: &Pubkey) -> Pubkey {
    _find_lst_state_list_address(program_id.clone()).0
}

pub fn get_lst_state_list(rpc: &RpcClient, lst_state_list_addr: &Pubkey) -> Result<Vec<LstState>> {
    let lst_state_list_acc = rpc.get_account(&lst_state_list_addr)?;
    let lst_state_list = try_lst_state_list(&lst_state_list_acc.data)?;
    Ok(lst_state_list.to_vec())
}

pub fn find_and_get_lst_state_list(rpc: &RpcClient, program_id: &Pubkey) -> Result<Vec<LstState>> {
    let lst_state_list_addr = find_lst_state_list_address(program_id);
    get_lst_state_list(rpc, &lst_state_list_addr)
}

// Mint state helper

pub fn get_reserves_account(
    rpc: &RpcClient,
    reserves_addr: &Pubkey
) -> Result<spl_token_2022::state::Account> {
    let reserves_acc = rpc.get_account(&reserves_addr)?;
    let state =
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            &reserves_acc.data
        )?;
    Ok(state.base)
}

pub fn get_protocol_fee_accumulator_account(
    rpc: &RpcClient,
    protocol_fee_accum_addr: &Pubkey
) -> Result<spl_token_2022::state::Account> {
    let protocol_fee_accum_acc = rpc.get_account(&protocol_fee_accum_addr)?;
    let state =
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
            &protocol_fee_accum_acc.data
        )?;
    Ok(state.base)
}

// Pool state helper

pub fn lp_token_mint_addr(pool_state: &PoolState) -> Result<Pubkey> {
    Ok(pool_state.lp_token_mint)
}

// Lst State helper

pub fn find_pool_reserves_address(
    lst_state: &LstState,
    pool_state_addr: &Pubkey,
    token_program: &Pubkey
) -> Result<Pubkey> {
    let reserves_addr = create_pool_reserves_address_with_pool_state_id(
        pool_state_addr.clone(),
        lst_state,
        token_program.clone()
    )?;
    Ok(reserves_addr)
}

pub fn protocol_fee_accum_address(
    lst_state: &LstState,
    protocol_fee_id: &Pubkey,
    token_program: &Pubkey
) -> Result<Pubkey> {
    let protocol_fee_accum_addr = create_protocol_fee_accumulator_address_with_protocol_fee_id(
        protocol_fee_id.clone(),
        lst_state,
        token_program.clone()
    )?;
    Ok(protocol_fee_accum_addr)
}

#[cfg(test)]
mod tests {
    use crate::{ controller, state::find_and_get_lst_state_list };

    use super::find_and_get_pool_state;

    #[test]
    fn test_lst_state() {
        let program_id = controller::ID;
        let rpc = solana_client::rpc_client::RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string()
        );

        let pool_state = find_and_get_pool_state(&rpc, &program_id).unwrap();
        println!("Pool State: {:?}", pool_state.total_sol_value);

        let lst_state_list = find_and_get_lst_state_list(&rpc, &program_id).unwrap();
        for lst_state in lst_state_list {
            println!("LST Mint: {:?}", lst_state.mint);
            println!("  sol_value: {:?}", lst_state.sol_value);
        }
    }
}

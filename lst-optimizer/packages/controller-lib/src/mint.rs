use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use anyhow::Result;

pub fn get_mint(rpc: &RpcClient, token_addr: &Pubkey) -> Result<spl_token_2022::state::Mint> {
    let lp_mint_acc = rpc.get_account(&token_addr)?;
    let state =
        spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
            &lp_mint_acc.data
        )?;
    Ok(state.base)
}

pub fn get_owner(rpc: &RpcClient, token_addr: &Pubkey) -> Result<Pubkey> {
    let lp_mint_acc = rpc.get_account(&token_addr)?;
    Ok(lp_mint_acc.owner)
}

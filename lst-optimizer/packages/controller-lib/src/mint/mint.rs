use solana_sdk::pubkey::Pubkey;
use anyhow::Result;

use crate::controller::ControllerClient;

// Mint state helper

pub trait MintAccountQuery {
    fn mint(&self, pubkey: &Pubkey) -> Result<spl_token_2022::state::Mint>;
    fn owner(&self, pubkey: &Pubkey) -> Result<Pubkey>;
}

impl MintAccountQuery for ControllerClient {
    fn mint(&self, pubkey: &Pubkey) -> Result<spl_token_2022::state::Mint> {
        let rpc = self.rpc_client();
        let lp_mint_acc = rpc.get_account(pubkey)?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
                &lp_mint_acc.data
            )?;
        Ok(state.base)
    }

    fn owner(&self, pubkey: &Pubkey) -> Result<Pubkey> {
        let rpc = self.rpc_client();
        let lp_mint_acc = rpc.get_account(pubkey)?;
        Ok(lp_mint_acc.owner)
    }
}

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use anyhow::Result;

use solana_readonly_account::{ ReadonlyAccountOwner, ReadonlyAccountPubkey };

/// A mint and its owner token program
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MintWithTokenProgram {
    /// The mint's pubkey
    pub pubkey: Pubkey,

    /// The mint's owner token program
    pub token_program: Pubkey,
}

impl ReadonlyAccountOwner for MintWithTokenProgram {
    fn owner(&self) -> &Pubkey {
        &self.token_program
    }
}

impl ReadonlyAccountPubkey for MintWithTokenProgram {
    fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }
}

// Mint state helper

pub trait MintAccountResolver {
    fn resolve_mint(&self, rpc: &RpcClient) -> Result<spl_token_2022::state::Mint>;
    fn resolve_owner(&self, rpc: &RpcClient) -> Result<Pubkey>;
}

impl MintAccountResolver for Pubkey {
    fn resolve_mint(&self, rpc: &RpcClient) -> Result<spl_token_2022::state::Mint> {
        let lp_mint_acc = rpc.get_account(&self)?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
                &lp_mint_acc.data
            )?;
        Ok(state.base)
    }

    fn resolve_owner(&self, rpc: &RpcClient) -> Result<Pubkey> {
        let lp_mint_acc = rpc.get_account(&self)?;
        Ok(lp_mint_acc.owner)
    }
}

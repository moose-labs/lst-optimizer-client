use anyhow::Result;
pub use s_controller_interface::{LstState, PoolState};
use s_controller_lib::{
    create_pool_reserves_address_with_pool_state_id,
    create_protocol_fee_accumulator_address_with_protocol_fee_id,
    find_lst_state_list_address as _find_lst_state_list_address,
    find_pool_state_address as _find_pool_state_address, try_lst_state_list, try_pool_state,
};
use solana_sdk::pubkey::Pubkey;

use crate::controller::ControllerClient;

#[async_trait::async_trait]
pub trait PoolQuery {
    async fn pool_state_address(&self, program_id: &Pubkey) -> Pubkey;
    async fn pool_state(&self, pool_state_addr: &Pubkey) -> Result<PoolState>;
    async fn pool_state_from_program_id(&self, program_id: &Pubkey) -> Result<PoolState>;
    async fn lst_state_list_address(&self, program_id: &Pubkey) -> Pubkey;
    async fn lst_state_list(&self, lst_state_list_addr: &Pubkey) -> Result<Vec<LstState>>;
    async fn lst_state_list_from_program_id(&self, program_id: &Pubkey) -> Result<Vec<LstState>>;

    async fn pool_reserves_address(
        &self,
        lst_state: &LstState,
        pool_state_addr: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Pubkey>;
    async fn pool_reserves_account(
        &self,
        reserves_address: &Pubkey,
    ) -> Result<spl_token_2022::state::Account>;

    async fn protocol_fee_accumulator_account(
        &self,
        protocol_fee_accum_addr: &Pubkey,
    ) -> Result<spl_token_2022::state::Account>;
    async fn protocol_fee_accumulator_address(
        &self,
        lst_state: &LstState,
        protocol_fee_id: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Pubkey>;
}

#[async_trait::async_trait]
impl PoolQuery for ControllerClient {
    async fn pool_state_address(&self, program_id: &Pubkey) -> Pubkey {
        _find_pool_state_address(program_id.clone()).0
    }

    async fn pool_state(&self, pool_state_addr: &Pubkey) -> Result<PoolState> {
        let pool_state_acc = self.rpc_client().get_account(&pool_state_addr).await?;
        let pool_state = try_pool_state(&pool_state_acc.data)?;
        Ok(*pool_state)
    }

    async fn pool_state_from_program_id(&self, program_id: &Pubkey) -> Result<PoolState> {
        let pool_state_addr = self.pool_state_address(program_id).await;
        self.pool_state(&pool_state_addr).await
    }

    async fn lst_state_list_address(&self, program_id: &Pubkey) -> Pubkey {
        _find_lst_state_list_address(program_id.clone()).0
    }

    async fn lst_state_list(&self, lst_state_list_addr: &Pubkey) -> Result<Vec<LstState>> {
        let lst_state_list_acc = self.rpc_client().get_account(&lst_state_list_addr).await?;
        let lst_state_list = try_lst_state_list(&lst_state_list_acc.data)?;
        Ok(lst_state_list.to_vec())
    }

    async fn lst_state_list_from_program_id(&self, program_id: &Pubkey) -> Result<Vec<LstState>> {
        let lst_state_list_addr = self.lst_state_list_address(program_id).await;
        self.lst_state_list(&lst_state_list_addr).await
    }

    // Mint state helper

    async fn pool_reserves_address(
        &self,
        lst_state: &LstState,
        pool_state_addr: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Pubkey> {
        let reserves_addr = create_pool_reserves_address_with_pool_state_id(
            pool_state_addr.clone(),
            lst_state,
            token_program.clone(),
        )?;
        Ok(reserves_addr)
    }

    async fn pool_reserves_account(
        &self,
        reserves_address: &Pubkey,
    ) -> Result<spl_token_2022::state::Account> {
        let reserves_acc = self.rpc_client().get_account(&reserves_address).await?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
                &reserves_acc.data
            )?;
        Ok(state.base)
    }

    async fn protocol_fee_accumulator_address(
        &self,
        lst_state: &LstState,
        protocol_fee_id: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Pubkey> {
        let protocol_fee_accum_addr = create_protocol_fee_accumulator_address_with_protocol_fee_id(
            protocol_fee_id.clone(),
            lst_state,
            token_program.clone(),
        )?;
        Ok(protocol_fee_accum_addr)
    }

    async fn protocol_fee_accumulator_account(
        &self,
        protocol_fee_accum_addr: &Pubkey,
    ) -> Result<spl_token_2022::state::Account> {
        let protocol_fee_accum_acc = self
            .rpc_client()
            .get_account(&protocol_fee_accum_addr)
            .await?;
        let state =
            spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(
                &protocol_fee_accum_acc.data
            )?;
        Ok(state.base)
    }
}

#[cfg(test)]
mod tests {
    use crate::controller;

    use super::*;

    #[tokio::test]
    async fn test_lst_state() {
        let program_id = controller::ID;
        let rpc = solana_client::nonblocking::rpc_client::RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
        );

        let controller = ControllerClient::new(rpc);
        let pool_state = controller
            .pool_state_from_program_id(&program_id)
            .await
            .unwrap();
        println!("Pool State: {:?}", pool_state.total_sol_value);

        let lst_state_list = controller
            .lst_state_list_from_program_id(&program_id)
            .await
            .unwrap();
        for lst_state in lst_state_list {
            println!("LST Mint: {:?}", lst_state.mint);
            println!("  sol_value: {:?}", lst_state.sol_value);
        }
    }
}

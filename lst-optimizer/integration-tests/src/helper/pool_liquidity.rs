use moose_utils::result::Result;
use s_controller_client::client::SControllerClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use tester::helper::instructions::s_controller::SController;

#[async_trait::async_trait]
pub trait PoolLstLiquidity {
    async fn add_lst_if_possible(
        &self,
        calculator_program: &Pubkey,
        mint: &Pubkey,
        admin: &Keypair,
    ) -> Result<()>;
}

#[async_trait::async_trait]
impl PoolLstLiquidity for SControllerClient {
    async fn add_lst_if_possible(
        &self,
        calculator_program: &Pubkey,
        mint: &Pubkey,
        admin: &Keypair,
    ) -> Result<()> {
        Ok(())
        // let mut is_lst_initialized = false;
        // let lst_list = self.get_lst_state_list().await;
        // if !lst_list.is_err() {
        //     is_lst_initialized = lst_list.unwrap().iter().any(|x| x.mint.eq(mint));
        // }

        // if !is_lst_initialized {
        //     self.add_lst(&mint, &calculator_program, admin)
        //         .await
        //         .unwrap();
        // }

        // Ok(())
    }
}

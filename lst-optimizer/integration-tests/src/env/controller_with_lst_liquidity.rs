use base_client::client::Client;
use flat_fee_client::client::FlatFeeClient;
use marinade_calculator_client::client::MarinadeCalculatorClient;
use moose_utils::result::Result;
use s_controller_client::client::SControllerClient;
use solana_sdk::{
    instruction::AccountMeta, signature::read_keypair_file, signer::Signer, system_instruction,
};
use spl_calculator_client::client::SplCalculatorClient;
use spl_token::instruction::sync_native;
use tester::{helper::instructions::s_controller::SController, utils::paths::get_deps_configs};

use crate::keys::{jitosol::JitoKeys, msol::MarinadeKeys, wsol::WsolKeys, LstKeys};

use super::controller_with_lst::new_controller_with_lst_list;

pub async fn new_controller_with_lst_liquidity() -> Result<(
    SControllerClient,
    FlatFeeClient,
    MarinadeCalculatorClient,
    SplCalculatorClient,
)> {
    let (s_controller_client, flat_fee_client, marinade_calculator_client, spl_calculator_client) =
        new_controller_with_lst_list().await?;

    let funder: solana_sdk::signature::Keypair =
        read_keypair_file(get_deps_configs("user1.json")).unwrap();
    let funder_pubkey = funder.pubkey();

    let pool_state = s_controller_client.get_pool_state().await?;

    let _ = s_controller_client
        .create_ata(&pool_state.lp_token_mint, &funder_pubkey)
        .await?;

    let funder_lp_token_account = s_controller_client
        .get_ata(&pool_state.lp_token_mint, &funder.pubkey())
        .await?;

    let lst_add_amount = 100_000_000_000; // 100 tokens
    let rpc = s_controller_client.rpc_client();

    // mSOL
    let (_, msol_mint) = MarinadeKeys::get_cal_program_and_mint();
    let funder_lst_token_account = s_controller_client
        .get_ata(&msol_mint, &funder_pubkey)
        .await?;
    s_controller_client
        .add_liquidity(
            &msol_mint,
            &funder_lst_token_account,
            &funder_lp_token_account,
            lst_add_amount,
            0,
            &MarinadeKeys::fetch_account_metas(rpc).await?,
            &[AccountMeta {
                pubkey: msol_mint,
                is_signer: false,
                is_writable: false,
            }],
        )
        .await?;

    let (_, jitosol_mint) = JitoKeys::get_cal_program_and_mint();
    let funder_lst_token_account = s_controller_client
        .get_ata(&jitosol_mint, &funder_pubkey)
        .await?;
    s_controller_client
        .add_liquidity(
            &jitosol_mint,
            &funder_lst_token_account,
            &funder_lp_token_account,
            lst_add_amount,
            0,
            &JitoKeys::fetch_account_metas(rpc).await?,
            &[AccountMeta {
                pubkey: jitosol_mint,
                is_signer: false,
                is_writable: false,
            }],
        )
        .await?;

    // Create wSOL account for funder first
    let (_, wsol_mint) = WsolKeys::get_cal_program_and_mint();
    let wsol_ata = s_controller_client
        .create_ata(&wsol_mint, &funder_pubkey)
        .await?;
    let transfer_ix = system_instruction::transfer(&funder_pubkey, &wsol_ata, 200_000_000_000);
    let sync_native_ix = sync_native(&spl_token::ID, &wsol_ata)?;
    s_controller_client
        .process_instructions(&[transfer_ix, sync_native_ix], &[])
        .await?;

    let funder_lst_token_account = s_controller_client
        .get_ata(&wsol_mint, &funder_pubkey)
        .await?;
    s_controller_client
        .add_liquidity(
            &wsol_mint,
            &funder_lst_token_account,
            &funder_lp_token_account,
            lst_add_amount,
            0,
            &WsolKeys::fetch_account_metas(rpc).await?,
            &[AccountMeta {
                pubkey: wsol_mint,
                is_signer: false,
                is_writable: false,
            }],
        )
        .await?;

    Ok((
        s_controller_client,
        flat_fee_client,
        marinade_calculator_client,
        spl_calculator_client,
    ))
}

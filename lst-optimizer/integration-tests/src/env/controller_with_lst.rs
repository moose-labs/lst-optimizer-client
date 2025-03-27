use flat_fee_client::client::FlatFeeClient;
use marinade_calculator_client::client::MarinadeCalculatorClient;
use moose_utils::result::Result;
use s_controller_client::client::SControllerClient;
use solana_sdk::signature::read_keypair_file;
use spl_calculator_client::client::SplCalculatorClient;
use tester::{helper::instructions::s_controller::SController, utils::paths::get_deps_configs};

use crate::keys::{jitosol::JitoKeys, msol::MarinadeKeys, wsol::WsolKeys, LstKeys};

use super::controller::setup_test_controller;

pub async fn new_controller_with_lst_list() -> Result<(
    SControllerClient,
    FlatFeeClient,
    MarinadeCalculatorClient,
    SplCalculatorClient,
)> {
    let (s_controller_client, flat_fee_client, marinade_calculator_client, spl_calculator_client) =
        setup_test_controller().await?;

    let admin = read_keypair_file(get_deps_configs("admin.json")).unwrap();

    let msol_mint = MarinadeKeys::get_lsl_mint();
    let msol_cal_program_id = MarinadeKeys::get_calculator_program_id();
    s_controller_client
        .add_lst(&msol_mint, &msol_cal_program_id, &admin)
        .await?;

    let jitosol_mint = JitoKeys::get_lsl_mint();
    let jitosol_cal_program_id = JitoKeys::get_calculator_program_id();
    s_controller_client
        .add_lst(&jitosol_mint, &jitosol_cal_program_id, &admin)
        .await?;

    let wsol_mint = WsolKeys::get_lsl_mint();
    let wsol_cal_program_id = WsolKeys::get_calculator_program_id();
    s_controller_client
        .add_lst(&wsol_mint, &wsol_cal_program_id, &admin)
        .await?;

    Ok((
        s_controller_client,
        flat_fee_client,
        marinade_calculator_client,
        spl_calculator_client,
    ))
}

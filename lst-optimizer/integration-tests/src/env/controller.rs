use flat_fee_client::client::FlatFeeClient;
use marinade_calculator_client::client::MarinadeCalculatorClient;
use moose_utils::result::Result;
use s_controller_client::client::SControllerClient;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signature::Signer;
use spl_calculator_client::client::SplCalculatorClient;
use tester::{
    helper::instructions::{
        flat_fee::FlatFee, marinade_calculator::MarinadeCalculator, s_controller::SController,
        spl_calculator::SplCalculator,
    },
    test_utils::{
        new_flat_fee_client, new_marinade_calculator_client, new_s_controller_client_with_keypair,
        new_spl_calculator_client,
    },
    utils::paths::get_deps_configs,
};

pub async fn setup_test_controller() -> Result<(
    SControllerClient,
    FlatFeeClient,
    MarinadeCalculatorClient,
    SplCalculatorClient,
)> {
    // initialize flat-fee contract.
    let (flat_fee_client, _) = new_flat_fee_client()?;
    flat_fee_client.initialize().await?;

    // initialize marinade calculator contract.
    let (marinade_calculator_client, initial_manager_keypair) = new_marinade_calculator_client()?;
    marinade_calculator_client.init_if_possible().await?;
    marinade_calculator_client
        .update_last_upgrade_slot(&initial_manager_keypair)
        .await?;

    // initialize spl calculator contract.
    let (spl_calculator_client, initial_manager_keypair) = new_spl_calculator_client()?;
    spl_calculator_client.init_if_possible().await?;
    spl_calculator_client
        .update_last_upgrade_slot(&initial_manager_keypair)
        .await?;

    // initialize s-controller contract.
    let (s_controller_client, initial_s_authority_keypair) =
        new_s_controller_client_with_keypair("user1.json")?; // user1.json is the payer

    s_controller_client
        .just_initialize(&initial_s_authority_keypair)
        .await?;

    let admin = read_keypair_file(get_deps_configs("admin.json"))?;

    s_controller_client
        .set_admin_if_not_match(&admin.pubkey(), &initial_s_authority_keypair)
        .await?;

    let user1 = read_keypair_file(get_deps_configs("user1.json"))?;

    s_controller_client
        .set_rebalance_authority_by_admin(&user1.pubkey(), &admin)
        .await?;

    Ok((
        s_controller_client,
        flat_fee_client,
        marinade_calculator_client,
        spl_calculator_client,
    ))
}

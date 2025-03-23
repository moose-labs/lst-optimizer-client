use crate::env::controller_with_lst_liquidity::new_controller_with_lst_liquidity;
use crate::env::lst_optimizer_app::new_lst_optimizer_app_with_quoter;
use crate::env::mockable_quoter_client::MockableQuoterClient;
use crate::keys::msol::MarinadeKeys;
use crate::keys::LstKeys;
use base_client::client::Client;
use lst_optimizer_std::pool::PoolRebalancable;
use lst_optimizer_std::types::amount_change::AmountChange;
use lst_optimizer_std::types::pool_allocation_changes::PoolAssetChange;
use moose_utils::result::Result;
use quoter_lib::typedefs::QuoterClient;
use s_controller_lib::{find_pool_reserves_address, FindLstPdaAtaKeys};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;
use tester::test_utils::TestValidator;
use tester::utils::paths::get_deps_configs;

async fn rebalance_asset_with_return_mul(repay_multiplier: u64) -> Result<()> {
    let _validator = TestValidator::new().await?;

    let (s_controller_client, _, _, _) = new_controller_with_lst_liquidity().await?;

    let user1 = read_keypair_file(get_deps_configs("user1.json")).unwrap();

    // Setup prepare the associated token accounts for wSOL and mSOL
    let setup_instructions: Vec<Instruction> = vec![];

    // Swap receives wSOL from the pool and sends mSOL to the pool
    let rebalance_amount = 10_000_000_000;
    let (_, msol_mint) = MarinadeKeys::get_cal_program_and_mint();
    let (pool_reserves, _) = find_pool_reserves_address(FindLstPdaAtaKeys {
        lst_mint: msol_mint,
        token_program: MarinadeKeys::get_token_program_id(),
    });
    let swap_instructions: Vec<Instruction> = vec![
        // We will receive "rebalance_amount" wSOL from the pool
        // We will simply send "rebalance_amount * 2" mSOL return to the pool
        spl_token::instruction::transfer(
            &MarinadeKeys::get_token_program_id(),
            &s_controller_client
                .get_ata(&msol_mint, &user1.pubkey())
                .await?,
            &pool_reserves,
            &user1.pubkey(),
            &[&user1.pubkey()],
            rebalance_amount * repay_multiplier + 1, // Add 1 to avoid transfer zero amount
        )?,
    ];

    // This test rebalances wSOL to mSOL with assertion
    // The test mock the quoter client to avoid dependency on the real quoter client
    let rpc = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::processed(),
    );
    let quoter_client = Box::new(
        MockableQuoterClient::from_parts(rpc)
            .with_setup_instructions(setup_instructions)
            .with_swap_instructions(swap_instructions),
    );

    let (optimizer, context, _) = new_lst_optimizer_app_with_quoter(quoter_client);

    // Increase "rebalance_amount" mSOL to the pool
    let pool_asset_change = PoolAssetChange::new(
        &msol_mint.to_string(),
        AmountChange::Increase(rebalance_amount),
    );

    optimizer
        .get_pool()
        .rebalance_asset(&context, &pool_asset_change)
        .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_with_return_mul_2() -> Result<()> {
    // Test rebalancing wSOL to mSOL with return multiplier 2
    rebalance_asset_with_return_mul(2).await
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_with_return_mul_0() -> Result<()> {
    // Test rebalancing wSOL to mSOL with return multiplier 0
    let ret = rebalance_asset_with_return_mul(0).await;
    assert!(ret.is_err());
    assert!(ret.err().unwrap().to_string().contains("0x12"));
    Ok(())
}

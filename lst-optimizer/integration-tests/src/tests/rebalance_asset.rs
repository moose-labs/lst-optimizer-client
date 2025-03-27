use crate::env::controller_with_lst_liquidity::new_controller_with_lst_liquidity;
use crate::env::lst_optimizer_app::new_lst_optimizer_app_with_quoter;
use crate::env::mockable_quoter_client::MockableQuoterClient;
use crate::keys::msol::MarinadeKeys;
use crate::keys::wsol::WsolKeys;
use crate::keys::LstKeys;
use base_client::client::Client;
use controller_lib::Pubkey;
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

async fn rebalance_asset_with_return(
    pool_asset_change: PoolAssetChange,
    return_mint: Pubkey,
    return_mint_token_program_id: Pubkey,
    return_amount: u64,
) -> Result<()> {
    let _validator = TestValidator::new().await?;

    let (s_controller_client, _, _, _) = new_controller_with_lst_liquidity().await?;

    let user1 = read_keypair_file(get_deps_configs("user1.json")).unwrap();

    // Setup prepare the associated token accounts for "pool_asset_change.mint" and "return_mint"
    let setup_instructions: Vec<Instruction> = vec![];

    // Swap receives "pool_asset_change.mint" from the pool and sends "return_mint" to the pool
    let (pool_reserves, _) = find_pool_reserves_address(FindLstPdaAtaKeys {
        lst_mint: return_mint,
        token_program: return_mint_token_program_id,
    });
    let swap_instructions: Vec<Instruction> = vec![
        // We will receive "rebalance_amount" wSOL from the pool
        // We will simply send "rebalance_amount * 2" mSOL return to the pool
        spl_token::instruction::transfer(
            &return_mint_token_program_id,
            &s_controller_client
                .get_ata(&return_mint, &user1.pubkey())
                .await?,
            &pool_reserves,
            &user1.pubkey(),
            &[&user1.pubkey()],
            return_amount,
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

    optimizer
        .get_pool()
        .rebalance_asset(&context, &pool_asset_change)
        .await?;

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_decreased_with_no_loss() -> Result<()> {
    let msol_mint = MarinadeKeys::get_lsl_mint();
    let pool_asset_change = PoolAssetChange::new(
        &msol_mint.to_string(),
        AmountChange::Decrease {
            lamports: 0,
            lst_amount: 10_000_000_000,
        },
    );

    let return_mint = WsolKeys::get_lsl_mint();
    let return_mint_token_program_id = WsolKeys::get_token_program_id();
    rebalance_asset_with_return(
        pool_asset_change,
        return_mint,
        return_mint_token_program_id,
        20_000_000_000,
    )
    .await
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_decreased_with_loss() -> Result<()> {
    let msol_mint = MarinadeKeys::get_lsl_mint();
    let pool_asset_change = PoolAssetChange::new(
        &msol_mint.to_string(),
        AmountChange::Decrease {
            lamports: 0,
            lst_amount: 10_000_000_000,
        },
    );

    let return_mint = WsolKeys::get_lsl_mint();
    let return_mint_token_program_id = WsolKeys::get_token_program_id();
    let ret = rebalance_asset_with_return(
        pool_asset_change,
        return_mint,
        return_mint_token_program_id,
        10_000_000_000, // should receive more than 10_000_000_000, because 1 mSOL = ~1.278235 wSOL
    )
    .await;

    assert!(ret.is_err());
    assert!(ret.err().unwrap().to_string().contains("0x12")); // PoolWouldLoseSolValue = 18
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_increased_with_no_loss() -> Result<()> {
    let msol_mint = MarinadeKeys::get_lsl_mint();
    let pool_asset_change = PoolAssetChange::new(
        &msol_mint.to_string(),
        AmountChange::Increase {
            lamports: 10_000_000_000,
            lst_amount: 0,
        },
    );

    let return_mint = MarinadeKeys::get_lsl_mint();
    let return_mint_token_program_id = MarinadeKeys::get_token_program_id();
    rebalance_asset_with_return(
        pool_asset_change,
        return_mint,
        return_mint_token_program_id,
        8_000_000_000, // 1 mSOL = ~1.278235 wSOL, then we should receive 10 SOL / 1.278235 = 7.825 mSOL at least
    )
    .await
}

#[tokio::test]
#[serial_test::serial]
async fn test_rebalance_asset_increased_with_loss() -> Result<()> {
    let msol_mint = MarinadeKeys::get_lsl_mint();
    let pool_asset_change = PoolAssetChange::new(
        &msol_mint.to_string(),
        AmountChange::Increase {
            lamports: 10_000_000_000,
            lst_amount: 0,
        },
    );

    let return_mint = MarinadeKeys::get_lsl_mint();
    let return_mint_token_program_id = MarinadeKeys::get_token_program_id();
    let ret = rebalance_asset_with_return(
        pool_asset_change,
        return_mint,
        return_mint_token_program_id,
        7_000_000_000, // 1 mSOL = ~1.278235 wSOL, then we should receive 10 SOL / 1.278235 = 7.825 mSOL at least
    )
    .await;

    assert!(ret.is_err());
    assert!(ret.err().unwrap().to_string().contains("0x12")); // PoolWouldLoseSolValue = 18
    Ok(())
}

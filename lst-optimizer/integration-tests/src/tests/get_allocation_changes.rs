use lst_optimizer_std::{
    allocator::{AllocationRatio, AllocationRatios},
    types::pool_allocation::MAX_ALLOCATION_BPS,
};
use moose_utils::result::Result;
use tester::test_utils::TestValidator;

use crate::{
    env::{
        controller_with_lst_liquidity::new_controller_with_lst_liquidity,
        lst_optimizer_app::new_lst_optimizer_app,
    },
    keys::{jitosol::JitoKeys, msol::MarinadeKeys, wsol::WsolKeys, LstKeys},
};

#[tokio::test]
#[serial_test::serial]
async fn test_get_allocation_changes() -> Result<()> {
    let _validator = TestValidator::new().await?;

    let _ = new_controller_with_lst_liquidity().await?;

    let (optimizer, context, _) = new_lst_optimizer_app();

    let (_, msol_mint) = MarinadeKeys::get_cal_program_and_mint();
    let (_, jitosol_mint) = JitoKeys::get_cal_program_and_mint();
    let (_, wsol_mint) = WsolKeys::get_cal_program_and_mint();

    let deallocation_ratios = AllocationRatios::new(vec![
        AllocationRatio::new(&msol_mint.to_string(), 0),
        AllocationRatio::new(&jitosol_mint.to_string(), 0),
        AllocationRatio::new(&wsol_mint.to_string(), MAX_ALLOCATION_BPS),
    ]);

    // get_pool_allocation_changes fetches the pool's reserves and calculates the changes via calculator
    let pool_allocation = optimizer
        .get_pool_allocation_changes(&context, deallocation_ratios)
        .await?;

    assert_eq!(pool_allocation.assets.len(), 3);
    assert!(pool_allocation
        .get_asset_changes(&msol_mint.to_string())
        .unwrap()
        .amount
        .is_decrease());

    assert!(pool_allocation
        .get_asset_changes(&jitosol_mint.to_string())
        .unwrap()
        .amount
        .is_decrease());

    assert!(pool_allocation
        .get_asset_changes(&wsol_mint.to_string())
        .unwrap()
        .amount
        .is_increase());

    Ok(())
}

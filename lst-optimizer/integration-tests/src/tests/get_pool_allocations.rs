use lst_optimizer_std::pool::PoolAllocable;
use moose_utils::result::Result;
use tester::test_utils::TestValidator;

use crate::env::{
    controller_with_lst_liquidity::new_controller_with_lst_liquidity,
    lst_optimizer_app::new_lst_optimizer_app,
};

#[tokio::test]
#[serial_test::serial]
async fn test_get_pool_allocations() -> Result<()> {
    let _validator = TestValidator::new().await?;

    let _ = new_controller_with_lst_liquidity().await?;

    let (optimizer, context, _) = new_lst_optimizer_app();

    let pool_allocation = optimizer.get_pool().get_allocation(&context).await?;

    for asset in pool_allocation.assets.iter() {
        assert_eq!(asset.reserves, 100_000_000_000);
        assert!(
            asset.lamports >= 100_000_000_000,
            "expected {} to be greater than or equal to 100_000_000_000, got {}",
            asset.mint,
            asset.lamports
        );
    }

    Ok(())
}

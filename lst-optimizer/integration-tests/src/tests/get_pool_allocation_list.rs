use lst_optimizer_std::pool::PoolAllocable;
use lst_optimizer_std::types::asset_repository::AssetRepositoryError;
use moose_utils::result::Result;
use tester::test_utils::TestValidator;

use crate::env::{
    controller_with_lst::new_controller_with_lst_list,
    lst_optimizer_app::{new_lst_optimizer_app, new_lst_optimizer_app_with_registry},
};

#[tokio::test]
#[serial_test::serial]
async fn test_get_pool_lst_list() -> Result<()> {
    let _validator = TestValidator::new().await?;

    let _ = new_controller_with_lst_list().await?;

    let (optimizer, context, _) = new_lst_optimizer_app();

    let pool_allocation = optimizer.get_pool().get_allocation(&context).await?;

    // Known assets are 3 in the registry and pool reserves
    // wSOL, mSOL, JitoSOL
    assert_eq!(pool_allocation.assets.len(), 3);

    for asset in pool_allocation.assets.iter() {
        assert_eq!(asset.lamports, 0);
    }

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_pool_allocations_mismatch_registry() -> Result<()> {
    let _validator = TestValidator::new().await?;

    let _ = new_controller_with_lst_list().await?;

    // Missing wrapped SOL in the registry
    let partial_registry = "integration-tests/registry_test_partial.toml";
    let (optimizer, context, _) = new_lst_optimizer_app_with_registry(partial_registry);

    let pool_allocation = optimizer.get_pool().get_allocation(&context).await;
    assert!(pool_allocation.is_err());
    assert_eq!(
        pool_allocation.err().unwrap().to_string(),
        AssetRepositoryError::AssetMintNotFound(
            "So11111111111111111111111111111111111111112".to_string()
        )
        .to_string()
    );

    Ok(())
}

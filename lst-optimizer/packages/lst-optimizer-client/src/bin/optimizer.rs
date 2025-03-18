use controller_lib::controller;
use lst_optimizer_client::{
    app::OptimizerApp,
    pool::{MaxPool, MaxPoolOptions},
};
use lst_optimizer_std::{
    helper::config::asset_repository_from_toml, logger::setup_global_logger,
    types::context::Context,
};

#[tokio::main]
async fn main() {
    if let Err(err) = setup_global_logger() {
        eprintln!("{:?}", err);
    }

    let rpc_url = "http://127.0.0.1:8899".to_string();
    let asset_repository = asset_repository_from_toml("./registry.toml").unwrap();
    let interval = std::time::Duration::from_secs(60 * 60 * 24 * 2);

    let context = Context::default();
    let program_id = controller::ID.to_string();
    let pool = MaxPool::new(
        &program_id,
        MaxPoolOptions {
            rpc_url,
            ..Default::default()
        },
    );

    let err = OptimizerApp::new(pool)
        .keep_rebalance(
            context
                .with_asset_repository(asset_repository)
                .with_payer("86naSVEnAUH1C9b4WktPqohydNhW5c1Tnt2foQqnZKb1".to_string()),
            interval,
        )
        .await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

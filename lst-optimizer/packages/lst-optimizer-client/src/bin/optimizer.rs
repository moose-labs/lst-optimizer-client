use controller_lib::controller;
use lst_optimizer_client::app::OptimizerApp;
use lst_optimizer_std::{
    helper::config::asset_repository_from_toml,
    logger::setup_global_logger,
    types::context::Context,
};

#[tokio::main]
async fn main() {
    if let Err(err) = setup_global_logger() {
        eprintln!("Error while setting up logger: {:?}", err);
    }

    let asset_repository = asset_repository_from_toml("./maxsol_short.toml").unwrap();
    // TODO: change interval to 2 day interval (~1 epoch)
    let interval = std::time::Duration::from_secs(6000);

    let context = Context::default();
    let program_id = controller::ID.to_string();
    let err = OptimizerApp::new(program_id).keep_rebalance(
        context
            .with_asset_repository(asset_repository)
            .with_payer("86naSVEnAUH1C9b4WktPqohydNhW5c1Tnt2foQqnZKb1".to_string()),
        interval
    ).await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

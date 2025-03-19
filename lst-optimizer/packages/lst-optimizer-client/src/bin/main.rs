use controller_lib::controller;
use lst_optimizer_client::{
    app::OptimizerApp,
    pool::{pool::MaxPool, typedefs::MaxPoolOptions},
    utils::get_registry_file,
};
use lst_optimizer_std::{
    helper::config::asset_repository_from_toml, logger::setup_global_logger,
    types::context::Context, utils::path::get_deps_accounts,
};
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::read_keypair_file;

#[tokio::main]
async fn main() {
    if let Err(err) = setup_global_logger() {
        eprintln!("{:?}", err);
    }

    let payer = read_keypair_file(get_deps_accounts("admin.json")).expect("Failed to read keypair");
    let asset_repository = asset_repository_from_toml(get_registry_file()).unwrap();
    let interval = std::time::Duration::from_secs(60 * 60 * 24 * 2);

    let context = Context::default();
    let program_id = controller::ID.to_string();

    let rpc_url = "http://127.0.0.1:8899".to_string();
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
                .with_payer(payer.pubkey().to_string()),
            interval,
        )
        .await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

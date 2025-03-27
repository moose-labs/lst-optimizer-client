use lst_optimizer_client::{
    app::OptimizerApp,
    pool::{pool::MaxPool, typedefs::MaxPoolOptions},
};
use lst_optimizer_std::{helper::config::asset_repository_from_toml, types::context::Context};
use lst_optimizer_utils::path::get_workspace_file;
use quoter_lib::{mock_quoter::MockQuoterClient, typedefs::QuoterClient};
use solana_sdk::signature::{read_keypair_file, Keypair};
use tester::utils::paths::get_deps_configs;

pub fn new_lst_optimizer_app() -> (OptimizerApp, Context, Keypair) {
    new_lst_optimizer_app_custom(
        "integration-tests/registry_test.toml",
        Box::new(MockQuoterClient::new()),
    )
}

pub fn new_lst_optimizer_app_with_registry(registry: &str) -> (OptimizerApp, Context, Keypair) {
    new_lst_optimizer_app_custom(registry, Box::new(MockQuoterClient::new()))
}

pub fn new_lst_optimizer_app_with_quoter(
    quoter_client: Box<dyn QuoterClient>,
) -> (OptimizerApp, Context, Keypair) {
    new_lst_optimizer_app_custom("integration-tests/registry_test.toml", quoter_client)
}

pub fn new_lst_optimizer_app_custom(
    registry: &str,
    quoter_client: Box<dyn QuoterClient>,
) -> (OptimizerApp, Context, Keypair) {
    let admin = read_keypair_file(get_deps_configs("user1.json")).unwrap();

    let url = "http://localhost:8899";

    let asset_repository = asset_repository_from_toml(get_workspace_file(registry)).unwrap();

    let context = Context::default()
        .with_payer(admin.insecure_clone())
        .with_asset_repository(asset_repository);

    let pool: lst_optimizer_client::pool::pool::MaxPool = MaxPool::new(
        controller_lib::program::localnet::ID,
        quoter_client,
        MaxPoolOptions {
            rpc_url: url.to_string(),
            ..Default::default()
        },
    );

    (OptimizerApp::new(pool), context, admin)
}

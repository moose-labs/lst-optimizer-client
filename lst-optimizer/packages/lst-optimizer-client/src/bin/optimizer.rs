use clap::Parser;
use jupiter_lib::quoter::JupiterQuoterClient;
use lst_optimizer_client::{
    app::OptimizerApp,
    pool::{pool::MaxPool, typedefs::MaxPoolOptions},
    utils::{args::AppArgs, path::get_registry_file},
};
use lst_optimizer_std::{helper::config::asset_repository_from_toml, types::context::Context};
use lst_optimizer_utils::{logger::setup_global_logger, path::get_deps_configs};
use solana_sdk::signer::keypair::read_keypair_file;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    if let Err(err) = setup_global_logger() {
        eprintln!("{:?}", err);
    }

    let payer =
        read_keypair_file(get_deps_configs(args.keypair.as_str())).expect("Failed to read keypair");
    let asset_repository =
        asset_repository_from_toml(get_registry_file()).expect("Failed to read asset repository");

    let context = Context::default();
    let program_id = controller_lib::program::mainnet::ID;

    let rpc_url = args.url;
    let jupiter_quoter_client = JupiterQuoterClient::new(&rpc_url);
    let pool = MaxPool::new(
        program_id,
        Box::new(jupiter_quoter_client),
        MaxPoolOptions {
            rpc_url,
            minimum_rebalance_lamports: args.minimum_rebalance_lamports,
            ..Default::default()
        },
    );

    let err = OptimizerApp::new(pool)
        .keep_rebalance(
            context
                .with_asset_repository(asset_repository)
                .with_payer(payer),
            std::time::Duration::from_secs(args.interval),
        )
        .await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

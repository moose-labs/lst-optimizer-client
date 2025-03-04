use lst_optimizer_client::app::OptimizerApp;
use lst_optimizer_std::{ logger::setup_global_logger, types::asset::Asset };

#[tokio::main]
async fn main() {
    if let Err(err) = setup_global_logger() {
        eprintln!("Error while setting up logger: {:?}", err);
    }

    let symbols = vec![
        Asset::new_with_weight("jupsol", 1.0),
        Asset::new_with_weight("inf", 1.0),
        Asset::new_with_weight("jitosol", 0.5),
        Asset::new_with_weight("hsol", 0.5)
    ];
    let interval = std::time::Duration::from_secs(6000);

    let err = OptimizerApp::new().keep_rebalance(&symbols, interval).await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

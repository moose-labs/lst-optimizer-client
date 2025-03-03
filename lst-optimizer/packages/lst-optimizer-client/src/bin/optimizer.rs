use lst_optimizer_client::app::base_app::OptimizerApp;
use lst_optimizer_std::{ logger::setup_global_logger, types::weighted_symbol::WeightedSymbol };
use rust_decimal::Decimal;

#[tokio::main]
async fn main() {
    if let Err(err) = setup_global_logger() {
        eprintln!("Error while setting up logger: {:?}", err);
    }

    let symbols = vec![
        WeightedSymbol {
            symbol: "jupsol".to_string(),
            weight: Decimal::from(1),
        },
        WeightedSymbol {
            symbol: "inf".to_string(),
            weight: Decimal::from(1),
        },
        WeightedSymbol {
            symbol: "jitosol".to_string(),
            weight: Decimal::new(5, 1),
        },
        WeightedSymbol {
            symbol: "hsol".to_string(),
            weight: Decimal::new(5, 1),
        }
    ];
    let interval = std::time::Duration::from_secs(6000);

    let err = OptimizerApp::new().keep_rebalance(&symbols, interval).await;
    if let Err(err) = err {
        eprintln!("{:?}", err);
    }
}

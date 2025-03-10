use controller_lib::calculator::CalculatorType;
use lst_optimizer_std::types::asset::Asset;
use anyhow::Result;

pub fn pool_to_calculator_type(asset: &Asset) -> Result<CalculatorType> {
    let program = asset.program.clone().to_lowercase();
    match program.as_str() {
        "lido" => Ok(CalculatorType::Lido),
        "marinade" => Ok(CalculatorType::Marinade),
        "wsol" => Ok(CalculatorType::Wsol),
        "spl" | "sanctumspl" | "sanctumsplmulti" => {
            if asset.pool.is_none() {
                return Err(anyhow::anyhow!("expect pool for asset {}", asset.mint));
            }
            let pool = asset.pool.clone().unwrap();
            match program.as_str() {
                "spl" => Ok(CalculatorType::Spl(pool)),
                "sanctumspl" => Ok(CalculatorType::SanctumSpl(pool)),
                "sanctumsplmulti" => Ok(CalculatorType::SanctumSplMulti(pool)),
                _ => Err(anyhow::anyhow!("invalid calculator program type for {}", asset.symbol)),
            }
        }
        _ => Err(anyhow::anyhow!("invalid calculator program type for {}", asset.symbol)),
    }
}

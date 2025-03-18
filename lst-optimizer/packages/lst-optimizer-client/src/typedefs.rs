use anyhow::Result;
use controller_lib::calculator::typedefs::CalculatorType;
use lst_optimizer_std::types::asset::Asset;

pub fn pool_to_calculator_type(asset: &Asset) -> Result<CalculatorType> {
    let pool_info = asset.pool.clone();
    if pool_info.is_none() {
        return Err(anyhow::anyhow!("expect pool for asset {}", asset.mint));
    }

    let pool_info = pool_info.unwrap();
    let program = pool_info.program.clone().to_lowercase();
    match program.as_str() {
        "lido" => Ok(CalculatorType::Lido),
        "marinade" => Ok(CalculatorType::Marinade),
        "wsol" => Ok(CalculatorType::Wsol),
        "spl" | "sanctumspl" | "sanctumsplmulti" => {
            let pool: Option<String> = pool_info.pool.clone();
            if pool.is_none() {
                return Err(anyhow::anyhow!(
                    "expect pool address for asset {}",
                    asset.symbol
                ));
            }

            let pool = pool.unwrap();
            match program.as_str() {
                "spl" => Ok(CalculatorType::Spl(pool)),
                "sanctumspl" => Ok(CalculatorType::SanctumSpl(pool)),
                "sanctumsplmulti" => Ok(CalculatorType::SanctumSplMulti(pool)),
                _ => Err(anyhow::anyhow!(
                    "invalid calculator program type for {}",
                    asset.symbol
                )),
            }
        }
        _ => Err(anyhow::anyhow!(
            "invalid calculator program type for {}",
            asset.symbol
        )),
    }
}

use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub enum CalculatorType {
    Lido,
    Marinade,
    Wsol,
    Spl(String),
    SanctumSpl(String),
    SanctumSplMulti(String),
}

// Metadata for the common accounts used by all calculators

pub fn calculator_program_id(calculator_type: &CalculatorType) -> Pubkey {
    match calculator_type {
        CalculatorType::Lido => lido_calculator_lib::program::ID,
        CalculatorType::Marinade => marinade_calculator_lib::program::ID,
        CalculatorType::Wsol => wsol_calculator_lib::program::ID,
        CalculatorType::Spl(_) => spl_calculator_lib::program::ID,
        CalculatorType::SanctumSpl(_) => spl_calculator_lib::sanctum_spl_sol_val_calc_program::ID,
        CalculatorType::SanctumSplMulti(_) => {
            spl_calculator_lib::sanctum_spl_multi_sol_val_calc_program::ID
        }
    }
}

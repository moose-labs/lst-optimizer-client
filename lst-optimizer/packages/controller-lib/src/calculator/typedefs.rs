use borsh::BorshDeserialize;
use data_encoding::BASE64;
use sanctum_token_ratio::U64ValueRange;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionReturnData;
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum CalculatorType {
    Lido,
    Marinade,
    Wsol,
    Spl(String),
    SanctumSpl(String),
    SanctumSplMulti(String),
}

// Helper function to parse the return data

pub fn parse_u64_value_range_return_data(
    return_data: &UiTransactionReturnData
) -> Result<U64ValueRange> {
    let UiTransactionReturnData { data: (data_str, _), .. } = return_data;
    let data = BASE64.decode(data_str.as_bytes())?;
    let range = U64ValueRange::deserialize(&mut data.as_ref())?;
    Ok(range)
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

#[cfg(test)]
mod tests {
    use solana_transaction_status::UiReturnDataEncoding;

    use super::*;

    #[test]
    fn test_parse_u64_value_range_return_data() {
        let data = UiTransactionReturnData {
            program_id: String::new(),
            data: ("hW/KLgAAAACJb8ouAAAAAA==".to_string(), UiReturnDataEncoding::Base64),
        };
        let val = parse_u64_value_range_return_data(&data).unwrap();
        assert_eq!(val.get_min(), 785018757);
        assert_eq!(val.get_max(), 785018761);
    }
}

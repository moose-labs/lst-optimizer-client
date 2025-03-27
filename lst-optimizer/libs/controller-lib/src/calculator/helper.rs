use anyhow::Result;
use borsh::BorshDeserialize;
use data_encoding::BASE64;
use sanctum_token_ratio::U64ValueRange;
use solana_transaction_status::UiTransactionReturnData;

// Helper function to parse the return data

pub fn parse_u64_value_range_return_data(
    return_data: &UiTransactionReturnData,
) -> Result<U64ValueRange> {
    let UiTransactionReturnData {
        data: (data_str, _),
        ..
    } = return_data;
    let data = BASE64.decode(data_str.as_bytes())?;
    let range = U64ValueRange::deserialize(&mut data.as_ref())?;
    Ok(range)
}

#[cfg(test)]
mod tests {
    use solana_transaction_status::UiReturnDataEncoding;

    use super::*;

    #[test]
    fn test_parse_u64_value_range_return_data() {
        let data = UiTransactionReturnData {
            program_id: String::new(),
            data: (
                "hW/KLgAAAACJb8ouAAAAAA==".to_string(),
                UiReturnDataEncoding::Base64,
            ),
        };
        let val = parse_u64_value_range_return_data(&data).unwrap();
        assert_eq!(val.get_min(), 785018757);
        assert_eq!(val.get_max(), 785018761);
    }

    #[test]
    fn test_parse_u64_value_range_return_data_2() {
        let data = UiTransactionReturnData {
            program_id: String::new(),
            data: (
                "yBfAHgAAAADIF8AeAAAAAA==".to_string(),
                UiReturnDataEncoding::Base64,
            ),
        };
        let val = parse_u64_value_range_return_data(&data).unwrap();
        assert_eq!(val.get_min(), 0);
        assert_eq!(val.get_max(), 0);
    }
}

/// Represents a change in the amount of an item in (lamports, lst amount).
#[derive(Debug, Clone, PartialEq)]
pub enum AmountChange {
    Increase { lamports: u64, lst_amount: u64 },
    Decrease { lamports: u64, lst_amount: u64 },
}

impl AmountChange {
    pub fn is_increase(&self) -> bool {
        match self {
            AmountChange::Increase { .. } => true,
            _ => false,
        }
    }

    pub fn is_decrease(&self) -> bool {
        match self {
            AmountChange::Decrease { .. } => true,
            _ => false,
        }
    }
}

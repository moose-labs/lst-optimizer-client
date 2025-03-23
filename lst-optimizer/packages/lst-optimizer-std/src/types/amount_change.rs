#[derive(Debug, Clone, PartialEq)]
pub enum AmountChange {
    Increase(u64),
    Decrease(u64),
}

impl AmountChange {
    pub fn is_increase(&self) -> bool {
        match self {
            AmountChange::Increase(_) => true,
            _ => false,
        }
    }

    pub fn is_decrease(&self) -> bool {
        match self {
            AmountChange::Decrease(_) => true,
            _ => false,
        }
    }
}

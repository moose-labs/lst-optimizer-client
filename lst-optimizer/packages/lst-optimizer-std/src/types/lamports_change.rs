/// Represents a change in the amount of an item in (lamports, lst amount).
#[derive(Debug, Clone, PartialEq)]
pub enum LamportsChange {
    Increase(u64),
    Decrease(u64),
}

impl LamportsChange {
    pub fn is_increase(&self) -> bool {
        match self {
            LamportsChange::Increase(_) => true,
            _ => false,
        }
    }

    pub fn is_decrease(&self) -> bool {
        match self {
            LamportsChange::Decrease(_) => true,
            _ => false,
        }
    }

    pub fn get_lamports(&self) -> u64 {
        match self {
            LamportsChange::Increase(lamports) => *lamports,
            LamportsChange::Decrease(lamports) => *lamports,
        }
    }
}

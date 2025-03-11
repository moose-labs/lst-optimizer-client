#[derive(Debug, Clone, PartialEq)]
pub enum AmountChange {
    Increase(u64),
    Decrease(u64),
}

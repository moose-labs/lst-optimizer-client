use super::pool::MaxPool;

pub trait PoolRebalancable {
    fn rebalance(&self);
}

impl PoolRebalancable for MaxPool {
    fn rebalance(&self) {
        // rebalance logic
    }
}

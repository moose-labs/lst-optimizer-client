from .allocator import *
from constant.whitelist_lst import WeightedSymbol


class SingleAllocator(Allocator):
    """
    The SingleAllocator class to allocate
    """

    def __init__(
        self,
    ):
        pass

    def with_data(self, data) -> Allocator:
        assert isinstance(data, str), "data must be a string symbol"
        self.data = data
        return self

    def get_allocations(self, epoch: int) -> Allocations:
        self.assert_data_exists()
        return Allocations(epoch, [Allocation(self.data, Allocations.MAX_RATIO_BPS)])

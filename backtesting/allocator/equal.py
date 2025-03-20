from .allocator import *
from constant.whitelist_lst import WeightedSymbol


class EqualAllocator(Allocator):
    """
    The EqualAllocator class to allocate equally
    """

    def __init__(
        self,
    ):
        pass

    def with_data(self, data) -> Allocator:
        assert isinstance(data, list), "data must be a list of symbol"
        self.data = data
        return self

    def get_allocations(self, epoch: int) -> Allocations:
        self.assert_data_exists()
        weighted_symbols: list[WeightedSymbol] = self.data
        each_weight = Allocations.MAX_RATIO_BPS / len(weighted_symbols)
        return Allocations(
            epoch,
            [
                Allocation(weighted_symbol.symbol, each_weight)
                for weighted_symbol in weighted_symbols
            ],
        )

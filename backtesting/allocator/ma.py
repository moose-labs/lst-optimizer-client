from .allocator import *
from datasource.types import HistoricalApy
from dataclasses import dataclass
import pandas as pd


@dataclass
class MaHistoricalData:
    symbol: str
    historical_apys: list[HistoricalApy]


@dataclass
class SortedMa:
    symbol: str
    ma: float


class MaAllocator(Allocator):
    """
    The MaAllocator(5) class to allocate
    """

    def __init__(
        self,
    ):
        pass

    def with_data(self, data) -> Allocator:
        assert isinstance(data, list), "data must be a list of MaHistoricalData"
        self.data = data
        return self

    def get_allocations(self, epoch: int) -> Allocations:
        self.assert_data_exists()
        sorted_emas = self.get_sorted_ma()
        return Allocations(epoch, self.allocate_equal(sorted_emas))

    # internal

    def allocate_equal(self, sorted_emas: list[SortedMa], limit=6) -> list[Allocation]:
        """
        Allocate equally to the top n ma

        Args:
            sorted_emas (list[SortedEma]): The sorted ma
            limit (int, optional): The limit of ma to allocate. Defaults to 8.

        Returns:
            list[Allocation]: The allocations
        """
        if len(sorted_emas) > limit:
            sorted_emas = sorted_emas[:limit]
        return [
            Allocation(x.symbol, Allocations.MAX_RATIO_BPS / len(sorted_emas))
            for x in sorted_emas
        ]

    def get_sorted_ma(self) -> list[SortedMa]:
        """
        Get the sorted ma

        Returns:
            list[SortedMa]: The sorted ma
        """
        ma_historical_datas: list[MaHistoricalData] = self.data
        period = 5

        emas: list[SortedMa] = []
        for ma_historical_data in ma_historical_datas:
            ema = self.calculate_ma(ma_historical_data.historical_apys, period)
            if len(ema) == 0:
                continue
            emas.append(SortedMa(ma_historical_data.symbol, ema[-1]))

        return sorted(emas, key=lambda x: x.ma, reverse=True)

    def calculate_ma(
        self, historical_apys: list[HistoricalApy], period: int
    ) -> list[float]:
        """
        Calculate the ma

        Args:
            historical_apys (list[HistoricalApy]): The historical apys
            period (int): The period

        Returns:
            list[float]: The ma
        """
        data = [x.apy for x in historical_apys]
        ma_values = pd.Series(data).rolling(window=period).mean()
        return ma_values.to_list()

from .allocator import *
from datasource.types import HistoricalApy
from dataclasses import dataclass
import pandas as pd


@dataclass
class EmaHistoricalData:
    symbol: str
    historical_apys: list[HistoricalApy]


@dataclass
class SortedEma:
    symbol: str
    ema: float


class EmaAllocator(Allocator):
    """
    The EmaAllocator class to allocate

    Args:
        historical_apys_by_symbol (dict[str, list[HistoricalApy]]): The historical apys by symbol
        lookback_period (int, optional): The lookback period. Defaults to 5.
    """

    def __init__(
        self,
    ):
        pass

    def with_data(self, data) -> Allocator:
        assert isinstance(data, list), "data must be a list of EmaHistoricalData"
        self.data = data
        return self

    def get_allocations(self, epoch: int) -> Allocations:
        self.assert_data_exists()
        sorted_emas = self.get_sorted_emas()
        return Allocations(epoch, self.allocate_equal(sorted_emas))

    # internal

    def allocate_equal(self, sorted_emas: list[SortedEma], limit=6) -> list[Allocation]:
        """
        Allocate equally to the top n emas

        Args:
            sorted_emas (list[SortedEma]): The sorted emas
            limit (int, optional): The limit of emas to allocate. Defaults to 4.

        Returns:
            list[Allocation]: The allocations
        """
        if len(sorted_emas) > limit:
            sorted_emas = sorted_emas[:limit]
        return [
            Allocation(x.symbol, Allocations.MAX_RATIO_BPS / len(sorted_emas))
            for x in sorted_emas
        ]

    def get_sorted_emas(self, period=5) -> list[SortedEma]:
        """
        Get the sorted emas

        Returns:
            list[SortedEma]: The sorted emas
        """
        ema_historical_datas: list[EmaHistoricalData] = self.data

        emas: list[SortedEma] = []
        for ema_historical_data in ema_historical_datas:
            ema = self.calculate_ema(ema_historical_data.historical_apys, period)
            if len(ema) == 0:
                continue
            emas.append(SortedEma(ema_historical_data.symbol, ema[-1]))

        return sorted(emas, key=lambda x: x.ema, reverse=True)

    def calculate_ema(
        self, historical_apys: list[HistoricalApy], period: int
    ) -> list[float]:
        """
        Calculate the ema

        Args:
            historical_apys (list[HistoricalApy]): The historical apys
            period (int): The period

        Returns:
            list[float]: The ema
        """
        data = [x.apy for x in historical_apys]
        ema_values = pd.Series(data).ewm(span=period, adjust=False).mean()
        return ema_values.to_list()
